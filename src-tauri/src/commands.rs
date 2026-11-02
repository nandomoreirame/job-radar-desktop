//! IPC surface. Command bodies live in `*_impl` functions so tests call them
//! without a running Tauri app; the wrappers only unpack `State`.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

use crate::crawler::config::{CrawlSetup, load_crawl_setup};
use crate::crawler::runner::{
    RunGuard, RunTicket, config_error_code, crawl_with_real_sources, spawn_worker,
};
use crate::details::normalize_details;
use crate::domain::{BoardCard, CardDetails, CrawlerStatus, DiscardedJob, Outcome, Stage};
use crate::sqlite_store::SqliteStore;
use crate::store::{Store, StoreError};

pub struct AppState {
    pub store: Arc<dyn Store>,
    /// The crawl needs what only SQLite offers; tests with `MockStore` leave it empty.
    pub crawl_store: Option<Arc<SqliteStore>>,
    pub guard: RunGuard,
    pub config_path: PathBuf,
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

/// Refuses before any request: a second run, a missing store, or a bad config.
pub fn prepare_crawl(
    state: &AppState,
) -> Result<(RunTicket, CrawlSetup, Arc<SqliteStore>), String> {
    let ticket = state.guard.try_start().map_err(str::to_owned)?;
    let store = state.crawl_store.clone().ok_or("store_unavailable")?;
    let setup = load_crawl_setup(&state.config_path).map_err(|e| config_error_code(&e))?;
    Ok((ticket, setup, store))
}

#[tauri::command]
pub fn start_crawl(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let (ticket, setup, store) = prepare_crawl(&state)?;
    let notifier = app.clone();
    spawn_worker(
        ticket,
        move || {
            let progress = |text: &str| {
                // A closed window cannot hear it, and the crawl goes on regardless.
                let _ = app.emit("crawler-progress", text);
            };
            if let Err(error) =
                crawl_with_real_sources(&store, &setup, crate::clock::now(), &progress)
            {
                eprintln!("storage error: {}", error.detail());
            }
        },
        move || {
            let _ = notifier.emit("crawler-finished", ());
        },
    );
    Ok(())
}

pub fn crawler_status_impl(state: &AppState) -> Result<CrawlerStatus, String> {
    let run = match &state.crawl_store {
        Some(store) => store.last_run().map_err(report)?,
        None => None,
    };
    Ok(CrawlerStatus {
        running: state.guard.is_running(),
        run,
    })
}

#[tauri::command]
pub fn crawler_status(state: State<'_, AppState>) -> Result<CrawlerStatus, String> {
    crawler_status_impl(&state)
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

pub fn open_job_url_impl(
    state: &AppState,
    id: &str,
    open: impl Fn(&str) -> Result<(), String>,
) -> Result<(), String> {
    check_id(id)?;
    let url = state.store.card_url(id).map_err(report)?.ok_or("no_url")?;
    if !crate::details::is_http_url(&url) {
        return Err("invalid_url".into());
    }
    open(&url)
}

#[tauri::command]
pub fn open_job_url(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    open_job_url_impl(&state, &id, |url| {
        app.opener().open_url(url, None::<&str>).map_err(|e| {
            eprintln!("open url: {e}");
            "open_failed".into()
        })
    })
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

    fn state_with(store: impl Store + 'static) -> AppState {
        AppState {
            store: Arc::new(store),
            crawl_store: None,
            guard: RunGuard::default(),
            config_path: PathBuf::from("/data/none/crawler.json"),
        }
    }

    fn state() -> AppState {
        state_with(MockStore::new(sample_cards()))
    }

    fn crawl_state(config_path: PathBuf) -> AppState {
        AppState {
            crawl_store: Some(Arc::new(SqliteStore::open_in_memory().unwrap())),
            config_path,
            ..state()
        }
    }

    #[test]
    fn a_bad_config_refuses_the_run_and_frees_the_guard() {
        let state = crawl_state(PathBuf::from("/data/none/crawler.json"));
        assert_eq!(
            prepare_crawl(&state).err().as_deref(),
            Some("config_missing")
        );
        assert!(!state.guard.is_running());
        let status = crawler_status_impl(&state).unwrap();
        assert_eq!(
            status,
            CrawlerStatus {
                running: false,
                run: None
            }
        );
    }

    #[test]
    fn a_second_start_is_refused_while_one_runs() {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join("perfil.md");
        std::fs::write(&profile, include_str!("../tests/fixtures/perfil.md")).unwrap();
        let config = dir.path().join("crawler.json");
        let body = serde_json::json!({ "profile_path": profile }).to_string();
        std::fs::write(&config, body).unwrap();
        let state = crawl_state(config);
        let (ticket, setup, _) = prepare_crawl(&state).unwrap();
        assert!(!setup.skills.is_empty());
        assert!(crawler_status_impl(&state).unwrap().running);
        assert_eq!(
            prepare_crawl(&state).err().as_deref(),
            Some("already_running")
        );
        drop(ticket);
        assert!(prepare_crawl(&state).is_ok());
    }

    #[test]
    fn a_store_that_cannot_crawl_is_refused() {
        assert_eq!(
            prepare_crawl(&state()).err().as_deref(),
            Some("store_unavailable")
        );
        assert!(!state().guard.is_running());
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
        let state = state_with(FailingStore);
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
    fn opens_only_a_stored_http_url() {
        let state = state();
        let details = CardDetails {
            job_url: Some("https://a.example/1".into()),
            ..CardDetails::default()
        };
        update_card_impl(&state, "c01", details).unwrap();
        let opened = std::cell::RefCell::new(Vec::new());
        let open = |url: &str| {
            opened.borrow_mut().push(url.to_owned());
            Ok(())
        };
        open_job_url_impl(&state, "c01", open).unwrap();
        assert_eq!(*opened.borrow(), ["https://a.example/1"]);

        let cleared = state
            .store
            .update_card("c01", CardDetails::default())
            .unwrap();
        assert_eq!(cleared.job_url, None);
        assert_eq!(
            open_job_url_impl(&state, "c01", open),
            Err("no_url".to_string())
        );
        // A value that bypassed validation must not reach the OS.
        let bad = CardDetails {
            job_url: Some("javascript:alert(1)".into()),
            ..CardDetails::default()
        };
        state.store.update_card("c01", bad).unwrap();
        assert_eq!(
            open_job_url_impl(&state, "c01", open),
            Err("invalid_url".to_string())
        );
        assert_eq!(
            open_job_url_impl(&state, "", open),
            Err("invalid card id".to_string())
        );
        assert_eq!(opened.borrow().len(), 1);
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
