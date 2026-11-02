//! Persistence boundary. Commands and the UI only see `Store`; the SQLite
//! (default) and Postgres (self-hosted) backends implement it in later plans.

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

use crate::domain::DiscardedJob;
use crate::domain::{BoardCard, CardDetails, Outcome, Stage};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum StoreError {
    #[error("card not found: {0}")]
    NotFound(String),
    #[error("moving to closed requires an outcome")]
    OutcomeRequired,
    #[error("only closed cards take an outcome")]
    UnexpectedOutcome,
    #[error("store lock poisoned")]
    Poisoned,
    /// Database detail is logged server-side but never displayed in the webview.
    #[error("storage error")]
    Storage(String),
}

impl StoreError {
    pub fn detail(&self) -> String {
        match self {
            Self::Storage(detail) => detail.clone(),
            other => other.to_string(),
        }
    }
}

pub trait Store: Send + Sync {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError>;
    fn move_card(
        &self,
        id: &str,
        stage: Stage,
        outcome: Option<Outcome>,
    ) -> Result<BoardCard, StoreError>;
    /// `details` must already be normalized (see `details::normalize_details`).
    fn update_card(&self, id: &str, details: CardDetails) -> Result<BoardCard, StoreError>;
    fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError>;
    fn discard_card(&self, id: &str, reason: &str, at: &str) -> Result<(), StoreError>;
    fn restore_card(&self, id: &str) -> Result<BoardCard, StoreError>;
    fn delete_card(&self, id: &str, at: &str) -> Result<(), StoreError>;
    fn card_url(&self, id: &str) -> Result<Option<String>, StoreError>;
}

/// Checks the stage/outcome pair before any backend touches its data.
pub fn validate_move(stage: Stage, outcome: Option<Outcome>) -> Result<(), StoreError> {
    match (stage, outcome) {
        (Stage::Closed, None) => Err(StoreError::OutcomeRequired),
        (Stage::Closed, Some(_)) | (_, None) => Ok(()),
        (_, Some(_)) => Err(StoreError::UnexpectedOutcome),
    }
}

/// In-memory store seeded with fictional cards; changes last for the session.
#[cfg(test)]
pub struct MockStore {
    cards: Mutex<Vec<BoardCard>>,
    discards: Mutex<HashMap<String, (String, String)>>,
}

#[cfg(test)]
impl MockStore {
    pub fn new(cards: Vec<BoardCard>) -> Self {
        Self {
            cards: Mutex::new(cards),
            discards: Mutex::new(HashMap::new()),
        }
    }
}

#[cfg(test)]
impl Store for MockStore {
    fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError> {
        let discards = self.discards.lock().map_err(|_| StoreError::Poisoned)?;
        Ok(self
            .cards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .iter()
            .filter(|card| !discards.contains_key(&card.id))
            .cloned()
            .collect())
    }

    fn move_card(
        &self,
        id: &str,
        stage: Stage,
        outcome: Option<Outcome>,
    ) -> Result<BoardCard, StoreError> {
        validate_move(stage, outcome)?;
        let mut cards = self.cards.lock().map_err(|_| StoreError::Poisoned)?;
        let card = cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_owned()))?;
        card.stage = stage;
        card.outcome = outcome;
        Ok(card.clone())
    }

    fn update_card(&self, id: &str, details: CardDetails) -> Result<BoardCard, StoreError> {
        let mut cards = self.cards.lock().map_err(|_| StoreError::Poisoned)?;
        let card = cards
            .iter_mut()
            .find(|c| c.id == id)
            .ok_or_else(|| StoreError::NotFound(id.to_owned()))?;
        card.apply(details);
        Ok(card.clone())
    }

    fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError> {
        let discards = self.discards.lock().map_err(|_| StoreError::Poisoned)?;
        let cards = self.cards.lock().map_err(|_| StoreError::Poisoned)?;
        Ok(cards
            .iter()
            .filter_map(|card| {
                discards.get(&card.id).map(|(reason, at)| DiscardedJob {
                    id: card.id.clone(),
                    title: card.title.clone(),
                    company: card.company.clone(),
                    source: card.source.clone(),
                    location: card.location.clone(),
                    score: card.score,
                    reason: reason.clone(),
                    discarded_at: at.clone(),
                    job_url: card.job_url.clone(),
                    description: String::new(),
                })
            })
            .collect())
    }
    fn discard_card(&self, id: &str, reason: &str, at: &str) -> Result<(), StoreError> {
        if !self
            .cards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .iter()
            .any(|card| card.id == id)
        {
            return Err(StoreError::NotFound(id.into()));
        }
        self.discards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .insert(id.into(), (reason.into(), at.into()));
        Ok(())
    }
    fn restore_card(&self, id: &str) -> Result<BoardCard, StoreError> {
        self.discards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .remove(id)
            .ok_or_else(|| StoreError::NotFound(id.into()))?;
        self.cards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .iter()
            .find(|card| card.id == id)
            .cloned()
            .ok_or_else(|| StoreError::NotFound(id.into()))
    }
    fn delete_card(&self, id: &str, _at: &str) -> Result<(), StoreError> {
        self.discards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .remove(id)
            .ok_or_else(|| StoreError::NotFound(id.into()))?;
        self.cards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .retain(|card| card.id != id);
        Ok(())
    }
    fn card_url(&self, id: &str) -> Result<Option<String>, StoreError> {
        self.cards
            .lock()
            .map_err(|_| StoreError::Poisoned)?
            .iter()
            .find(|card| card.id == id)
            .map(|card| card.job_url.clone())
            .ok_or_else(|| StoreError::NotFound(id.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::CardDetails;
    use crate::mock_data::sample_cards;

    fn store() -> MockStore {
        MockStore::new(sample_cards())
    }

    #[test]
    fn mock_store_honors_the_store_contract() {
        crate::store_contract::check_all(|| {
            Box::new(MockStore::new(crate::mock_data::sample_cards()))
        });
    }

    #[test]
    fn storage_errors_hide_the_detail() {
        let error = StoreError::Storage("no such table: cards".into());
        assert_eq!(error.to_string(), "storage error");
        assert_eq!(error.detail(), "no such table: cards");
        assert_eq!(
            StoreError::NotFound("c01".into()).detail(),
            "card not found: c01"
        );
    }

    #[test]
    fn updating_applies_the_details_for_the_session() {
        let store = store();
        let details = CardDetails {
            notes: "Ligar na sexta".into(),
            recruiter_name: Some("Ana".into()),
            ..CardDetails::default()
        };
        let updated = store.update_card("c01", details).unwrap();
        assert_eq!(updated.notes, "Ligar na sexta");
        let listed = store.list_cards().unwrap();
        assert_eq!(
            listed
                .iter()
                .find(|c| c.id == "c01")
                .unwrap()
                .recruiter_name
                .as_deref(),
            Some("Ana")
        );
    }

    #[test]
    fn updating_an_unknown_card_is_not_found() {
        assert_eq!(
            store().update_card("nope", CardDetails::default()),
            Err(StoreError::NotFound("nope".into()))
        );
    }

    #[test]
    fn lists_every_seeded_card() {
        assert_eq!(store().list_cards().unwrap().len(), sample_cards().len());
    }

    #[test]
    fn moving_updates_the_stage_for_the_session() {
        let store = store();
        let moved = store.move_card("c01", Stage::Triage, None).unwrap();
        assert_eq!(moved.stage, Stage::Triage);
        let listed = store.list_cards().unwrap();
        assert_eq!(
            listed.iter().find(|c| c.id == "c01").unwrap().stage,
            Stage::Triage
        );
    }

    #[test]
    fn moving_to_closed_requires_an_outcome() {
        assert_eq!(
            store().move_card("c01", Stage::Closed, None),
            Err(StoreError::OutcomeRequired)
        );
    }

    #[test]
    fn an_outcome_outside_closed_is_rejected() {
        assert_eq!(
            store().move_card("c01", Stage::Offer, Some(Outcome::Hired)),
            Err(StoreError::UnexpectedOutcome)
        );
    }

    #[test]
    fn an_unknown_card_is_not_found() {
        assert_eq!(
            store().move_card("nope", Stage::Triage, None),
            Err(StoreError::NotFound("nope".into()))
        );
    }

    #[test]
    fn moving_out_of_closed_clears_the_outcome() {
        let store = store();
        store
            .move_card("c01", Stage::Closed, Some(Outcome::Discarded))
            .unwrap();
        let reopened = store.move_card("c01", Stage::Triage, None).unwrap();
        assert_eq!(reopened.outcome, None);
    }
}
