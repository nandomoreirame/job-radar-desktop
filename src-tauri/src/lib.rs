pub mod commands;
pub mod db_path;
pub mod details;
pub mod domain;
pub mod mock_data;
pub mod sqlite_store;
pub mod store;
#[cfg(test)]
mod store_contract;

use commands::AppState;
use sqlite_store::SqliteStore;
use std::path::Path;
use tauri::Manager;

pub fn open_store(path: &Path, debug: bool) -> Result<SqliteStore, String> {
    let fail = |cause: String| {
        format!(
            "Could not open the database at {}:\n{cause}",
            path.display()
        )
    };
    db_path::prepare(path).map_err(|e| fail(e.to_string()))?;
    let store = SqliteStore::open(path).map_err(|e| fail(e.detail()))?;
    if debug {
        store
            .seed_if_empty(&mock_data::sample_cards())
            .map_err(|e| fail(e.detail()))?;
    }
    Ok(store)
}

fn startup_failed(message: &str) -> ! {
    eprintln!("job-radar-desktop: {message}");
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("job-radar-desktop")
        .set_description(message)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_board,
            commands::move_card,
            commands::update_card,
            commands::window_controls
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => startup_failed(&error.to_string()),
    };
    let debug = cfg!(debug_assertions);
    let env_override = std::env::var(db_path::ENV_VAR).ok();
    let data_dir = match app.path().data_dir() {
        Ok(path) => path,
        Err(error) => startup_failed(&error.to_string()),
    };
    let path = db_path::db_path(env_override.as_deref(), &data_dir, debug);
    let store = match open_store(&path, debug) {
        Ok(store) => store,
        Err(message) => startup_failed(&message),
    };
    app.manage(AppState {
        store: Box::new(store),
    });
    app.run(|_, _| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    #[test]
    fn open_store_seeds_only_debug_and_only_when_empty() {
        let dir = tempfile::tempdir().unwrap();
        let release = open_store(&dir.path().join("release.db"), false).unwrap();
        assert!(release.list_cards().unwrap().is_empty());
        let debug_path = dir.path().join("debug.db");
        let debug = open_store(&debug_path, true).unwrap();
        assert_eq!(
            debug.list_cards().unwrap().len(),
            mock_data::sample_cards().len()
        );
        debug.move_card("c01", domain::Stage::Triage, None).unwrap();
        drop(debug);
        let again = open_store(&debug_path, true).unwrap();
        assert_eq!(again.list_cards().unwrap()[0].stage, domain::Stage::Triage);
    }
    #[test]
    fn open_store_reports_the_path_and_cause() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.db");
        std::fs::write(&path, b"this is not a sqlite database at all, just text").unwrap();
        let Err(message) = open_store(&path, true) else {
            panic!("a broken file must not open")
        };
        assert!(message.contains("broken.db"));
        assert!(message.contains("not a database"));
    }
}
