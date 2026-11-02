//! IPC surface. Command bodies live in `*_impl` functions so tests call them
//! without a running Tauri app; the wrappers only unpack `State`.

use tauri::State;

use crate::domain::{BoardCard, Outcome, Stage};
use crate::store::Store;

pub struct AppState {
    pub store: Box<dyn Store>,
}

/// Ids are short opaque strings; anything else from the webview is rejected.
pub const MAX_ID_LEN: usize = 64;

fn check_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err("invalid card id".into());
    }
    Ok(())
}

pub fn list_board_impl(state: &AppState) -> Result<Vec<BoardCard>, String> {
    state.store.list_cards().map_err(|e| e.to_string())
}

pub fn move_card_impl(
    state: &AppState,
    id: &str,
    stage: Stage,
    outcome: Option<Outcome>,
) -> Result<BoardCard, String> {
    check_id(id)?;
    state.store.move_card(id, stage, outcome).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_board(state: State<'_, AppState>) -> Result<Vec<BoardCard>, String> {
    list_board_impl(&state)
}

#[tauri::command]
pub fn move_card(
    state: State<'_, AppState>,
    id: String,
    stage: Stage,
    outcome: Option<Outcome>,
) -> Result<BoardCard, String> {
    move_card_impl(&state, &id, stage, outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_data::sample_cards;
    use crate::store::MockStore;

    fn state() -> AppState {
        AppState { store: Box::new(MockStore::new(sample_cards())) }
    }

    #[test]
    fn list_board_returns_the_seeded_cards() {
        assert_eq!(list_board_impl(&state()).unwrap(), sample_cards());
    }

    #[test]
    fn move_card_persists_the_new_stage() {
        let state = state();
        move_card_impl(&state, "c02", Stage::Triage, None).unwrap();
        let cards = list_board_impl(&state).unwrap();
        assert_eq!(cards.iter().find(|c| c.id == "c02").unwrap().stage, Stage::Triage);
    }

    #[test]
    fn move_card_rejects_an_empty_id() {
        assert_eq!(
            move_card_impl(&state(), "", Stage::Triage, None),
            Err("invalid card id".to_string())
        );
    }

    #[test]
    fn move_card_rejects_an_oversized_id() {
        let id = "x".repeat(MAX_ID_LEN + 1);
        assert_eq!(
            move_card_impl(&state(), &id, Stage::Triage, None),
            Err("invalid card id".to_string())
        );
    }

    #[test]
    fn move_card_reports_the_store_rule() {
        assert_eq!(
            move_card_impl(&state(), "c01", Stage::Closed, None),
            Err("moving to closed requires an outcome".to_string())
        );
    }
}
