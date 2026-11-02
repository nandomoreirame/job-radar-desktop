//! Shared behavior checks for every Store backend.

use crate::domain::{CardDetails, Outcome, Salary, Stage};
use crate::mock_data::sample_cards;
use crate::store::{Store, StoreError};

pub fn check_all(fresh: impl Fn() -> Box<dyn Store>) {
    lists_seeded_cards(&*fresh());
    moves_stage(&*fresh());
    requires_outcome(&*fresh());
    rejects_unexpected_outcome(&*fresh());
    clears_outcome(&*fresh());
    rejects_unknown_cards(&*fresh());
    updates_details(&*fresh());
}

fn lists_seeded_cards(store: &dyn Store) {
    assert_eq!(store.list_cards().unwrap(), sample_cards());
}
fn moves_stage(store: &dyn Store) {
    let moved = store.move_card("c01", Stage::Triage, None).unwrap();
    assert_eq!(moved.stage, Stage::Triage);
}
fn requires_outcome(store: &dyn Store) {
    assert_eq!(
        store.move_card("c01", Stage::Closed, None),
        Err(StoreError::OutcomeRequired)
    );
}
fn rejects_unexpected_outcome(store: &dyn Store) {
    assert_eq!(
        store.move_card("c01", Stage::Offer, Some(Outcome::Hired)),
        Err(StoreError::UnexpectedOutcome)
    );
}
fn clears_outcome(store: &dyn Store) {
    store
        .move_card("c01", Stage::Closed, Some(Outcome::Discarded))
        .unwrap();
    assert_eq!(
        store.move_card("c01", Stage::Triage, None).unwrap().outcome,
        None
    );
}
fn rejects_unknown_cards(store: &dyn Store) {
    assert_eq!(
        store.move_card("nope", Stage::Triage, None),
        Err(StoreError::NotFound("nope".into()))
    );
    assert_eq!(
        store.update_card("nope", CardDetails::default()),
        Err(StoreError::NotFound("nope".into()))
    );
}
fn updates_details(store: &dyn Store) {
    let before = store
        .list_cards()
        .unwrap()
        .into_iter()
        .find(|card| card.id == "c01")
        .unwrap();
    let details = CardDetails {
        salary: Some(Salary {
            pj: Some(30_000),
            clt: None,
        }),
        recruiter_name: Some("Ana".into()),
        notes: "Ligar na sexta".into(),
        ..CardDetails::default()
    };
    let mut expected = before;
    expected.apply(details.clone());
    assert_eq!(store.update_card("c01", details).unwrap(), expected);
    assert_eq!(
        store
            .list_cards()
            .unwrap()
            .into_iter()
            .find(|card| card.id == "c01")
            .unwrap(),
        expected
    );
}
