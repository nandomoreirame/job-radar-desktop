//! IPC surface. Command bodies live in `*_impl` functions so tests call them
//! without a running Tauri app; the wrappers only unpack `State`.

use std::env;
use tauri::State;

use crate::details::normalize_details;
use crate::domain::{BoardCard, CardDetails, DiscardedJob, Outcome, Stage};
use crate::store::{Store, StoreError};

pub struct AppState {
    pub store: Box<dyn Store>,
}

/// Ids are short opaque strings; anything else from the webview is rejected.
pub const MAX_ID_LEN: usize = 64;

pub fn is_tiling_wm(vars: impl IntoIterator<Item = (String, String)>) -> bool {
    vars.into_iter().any(|(key, value)| {
        matches!(
            key.as_str(),
            "NIRI_SOCKET" | "HYPRLAND_INSTANCE_SIGNATURE" | "SWAYSOCK" | "I3SOCK"
        ) && !value.is_empty()
            || matches!(key.as_str(), "XDG_CURRENT_DESKTOP" | "XDG_SESSION_DESKTOP")
                && value
                    .to_ascii_lowercase()
                    .split(':')
                    .any(|desktop| matches!(desktop, "niri" | "hyprland" | "sway" | "i3" | "river"))
    })
}

#[tauri::command]
pub fn window_controls() -> bool {
    is_tiling_wm(env::vars())
}

fn check_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return Err("invalid card id".into());
    }
    Ok(())
}

#[cfg(test)]
thread_local! {
    static LOGGED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn report(error: StoreError) -> String {
    if let StoreError::Storage(detail) = &error {
        #[cfg(test)]
        LOGGED.with_borrow_mut(|logged| logged.push(detail.clone()));
        eprintln!("storage error: {detail}");
    }
    error.to_string()
}

pub fn list_board_impl(state: &AppState) -> Result<Vec<BoardCard>, String> {
    state.store.list_cards().map_err(report)
}

pub fn move_card_impl(
    state: &AppState,
    id: &str,
    stage: Stage,
    outcome: Option<Outcome>,
) -> Result<BoardCard, String> {
    check_id(id)?;
    state.store.move_card(id, stage, outcome).map_err(report)
}

pub fn update_card_impl(
    state: &AppState,
    id: &str,
    details: CardDetails,
) -> Result<BoardCard, String> {
    check_id(id)?;
    let details = normalize_details(details)?;
    state.store.update_card(id, details).map_err(report)
}

#[tauri::command]
pub fn update_card(
    state: State<'_, AppState>,
    id: String,
    details: CardDetails,
) -> Result<BoardCard, String> {
    update_card_impl(&state, &id, details)
}

pub fn list_discarded_impl(state: &AppState) -> Result<Vec<DiscardedJob>, String> {
    state.store.list_discarded().map_err(report)
}
pub fn restore_card_impl(state: &AppState, id: &str) -> Result<BoardCard, String> {
    check_id(id)?;
    state.store.restore_card(id).map_err(report)
}
pub fn delete_card_impl(state: &AppState, id: &str, today: &str) -> Result<(), String> {
    check_id(id)?;
    state.store.delete_card(id, today).map_err(report)
}

#[tauri::command]
pub fn list_discarded(state: State<'_, AppState>) -> Result<Vec<DiscardedJob>, String> {
    list_discarded_impl(&state)
}
#[tauri::command]
pub fn restore_card(state: State<'_, AppState>, id: String) -> Result<BoardCard, String> {
    restore_card_impl(&state, &id)
}
#[tauri::command]
pub fn delete_card(state: State<'_, AppState>, id: String) -> Result<(), String> {
    delete_card_impl(&state, &id, &crate::clock::today())
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
    use crate::domain::{CardDetails, DiscardedJob};
    use crate::mock_data::sample_cards;
    use crate::store::MockStore;

    fn state() -> AppState {
        AppState {
            store: Box::new(MockStore::new(sample_cards())),
        }
    }

    struct FailingStore;

    impl Store for FailingStore {
        fn list_cards(&self) -> Result<Vec<BoardCard>, StoreError> {
            Err(StoreError::Storage("list SQL detail".into()))
        }
        fn move_card(
            &self,
            _: &str,
            _: Stage,
            _: Option<Outcome>,
        ) -> Result<BoardCard, StoreError> {
            Err(StoreError::Storage("move SQL detail".into()))
        }
        fn update_card(&self, _: &str, _: CardDetails) -> Result<BoardCard, StoreError> {
            Err(StoreError::Storage("update SQL detail".into()))
        }
        fn list_discarded(&self) -> Result<Vec<DiscardedJob>, StoreError> {
            Err(StoreError::Storage("discard SQL detail".into()))
        }
        fn discard_card(&self, _: &str, _: &str, _: &str) -> Result<(), StoreError> {
            Err(StoreError::Storage("discard SQL detail".into()))
        }
        fn restore_card(&self, _: &str) -> Result<BoardCard, StoreError> {
            Err(StoreError::Storage("restore SQL detail".into()))
        }
        fn delete_card(&self, _: &str, _: &str) -> Result<(), StoreError> {
            Err(StoreError::Storage("delete SQL detail".into()))
        }
        fn card_url(&self, _: &str) -> Result<Option<String>, StoreError> {
            Err(StoreError::Storage("url SQL detail".into()))
        }
    }

    #[test]
    fn every_command_logs_the_storage_detail_and_hides_it() {
        let state = AppState {
            store: Box::new(FailingStore),
        };
        let hidden = "storage error".to_string();
        assert_eq!(list_board_impl(&state).unwrap_err(), hidden);
        assert_eq!(
            move_card_impl(&state, "c01", Stage::Triage, None).unwrap_err(),
            hidden
        );
        assert_eq!(
            update_card_impl(&state, "c01", CardDetails::default()).unwrap_err(),
            hidden
        );
        assert_eq!(
            LOGGED.with_borrow(Vec::clone),
            ["list SQL detail", "move SQL detail", "update SQL detail"]
        );
    }

    #[test]
    fn update_card_persists_valid_details() {
        let state = state();
        let details = CardDetails {
            notes: "  anotar  ".into(),
            ..CardDetails::default()
        };
        assert_eq!(
            update_card_impl(&state, "c01", details).unwrap().notes,
            "anotar"
        );
    }

    #[test]
    fn update_card_reports_the_invalid_field() {
        let details = CardDetails {
            job_url: Some("javascript:alert(1)".into()),
            ..CardDetails::default()
        };
        assert_eq!(
            update_card_impl(&state(), "c01", details),
            Err("jobUrl: invalid_url".to_string())
        );
    }

    #[test]
    fn update_card_rejects_an_empty_id() {
        assert_eq!(
            update_card_impl(&state(), "", CardDetails::default()),
            Err("invalid card id".to_string())
        );
    }

    #[test]
    fn detects_tiling_window_managers() {
        for (key, value) in [
            ("NIRI_SOCKET", "/tmp/niri"),
            ("HYPRLAND_INSTANCE_SIGNATURE", "x"),
            ("SWAYSOCK", "x"),
            ("I3SOCK", "x"),
            ("XDG_CURRENT_DESKTOP", "river"),
        ] {
            assert!(is_tiling_wm([(key.to_string(), value.to_string())]));
        }
        assert!(!is_tiling_wm([(
            String::from("XDG_CURRENT_DESKTOP"),
            String::from("gnome")
        )]));
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
        assert_eq!(
            cards.iter().find(|c| c.id == "c02").unwrap().stage,
            Stage::Triage
        );
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
