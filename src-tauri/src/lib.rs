pub mod cli;
pub mod clock;
pub mod commands;
pub mod db_path;
pub mod details;
pub mod domain;
pub mod identity;
pub mod import;
#[cfg(test)]
mod mock_data;
pub mod sqlite_store;
pub mod store;
#[cfg(test)]
mod store_contract;

use commands::AppState;
use sqlite_store::SqliteStore;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use tauri::Manager;

pub fn open_store(path: &Path) -> Result<SqliteStore, String> {
    let fail = |cause: String| {
        format!(
            "Could not open the database at {}:\n{cause}",
            path.display()
        )
    };
    db_path::prepare(path).map_err(|e| fail(e.to_string()))?;
    let store = SqliteStore::open(path).map_err(|e| fail(e.detail()))?;
    Ok(store)
}

pub fn open_configured_store(
    env_override: Option<&OsStr>,
    data_dir: &Path,
    debug: bool,
) -> Result<SqliteStore, String> {
    open_store(&db_path::db_path(env_override, data_dir, debug))
}

fn can_show_dialog(var: impl Fn(&str) -> Option<OsString>) -> bool {
    ["WAYLAND_DISPLAY", "DISPLAY"]
        .iter()
        .any(|name| var(name).is_some_and(|value| !value.is_empty()))
}

fn startup_failed(message: &str) -> ! {
    eprintln!("job-radar-desktop: {message}");
    // rfd blocks forever when GTK cannot start, so a session with no display only gets stderr.
    if can_show_dialog(|name| std::env::var_os(name)) {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("job-radar-desktop")
            .set_description(message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
    std::process::exit(1);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<OsString> = std::env::args_os().skip(1).collect();
    match cli::parse(&args) {
        Ok(None) => {}
        Ok(Some(import_args)) => std::process::exit(run_import(&import_args)),
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(1);
        }
    }
    // Open the store before tauri::Builder: once tao owns the GTK main context,
    // the rfd dialog in startup_failed never runs and the process hangs.
    let data_dir = match dirs::data_dir() {
        Some(path) => path,
        None => startup_failed("Could not determine the user's data directory"),
    };
    let env_override = std::env::var_os(db_path::ENV_VAR);
    let store =
        match open_configured_store(env_override.as_deref(), &data_dir, cfg!(debug_assertions)) {
            Ok(store) => store,
            Err(message) => startup_failed(&message),
        };
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::list_board,
            commands::move_card,
            commands::update_card,
            commands::list_discarded,
            commands::restore_card,
            commands::delete_card,
            commands::open_job_url,
            commands::window_controls
        ])
        .build(tauri::generate_context!());
    let app = match app {
        Ok(app) => app,
        Err(error) => startup_failed(&error.to_string()),
    };
    app.manage(AppState {
        store: Box::new(store),
    });
    app.run(|_, _| {});
}

fn run_import(args: &cli::ImportArgs) -> i32 {
    let result = dirs::data_dir()
        .ok_or_else(|| "Could not determine the user's data directory".to_owned())
        .and_then(|data_dir| {
            let store = open_configured_store(
                std::env::var_os(db_path::ENV_VAR).as_deref(),
                &data_dir,
                cfg!(debug_assertions),
            )?;
            import::import_history(
                &store,
                &args.seen,
                args.candidates.as_deref(),
                &clock::today(),
            )
        });
    match result {
        Ok(report) => {
            println!("{}", cli::format_report(&report));
            0
        }
        Err(error) => {
            eprintln!("job-radar-desktop: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    #[test]
    fn open_store_never_seeds() {
        let dir = tempfile::tempdir().unwrap();
        for debug in [false, true] {
            let store = open_configured_store(Some(OsStr::new("")), dir.path(), debug).unwrap();
            assert!(store.list_cards().unwrap().is_empty());
        }
    }
    #[test]
    fn dialog_needs_a_display() {
        let var = |name: &str| (name == "WAYLAND_DISPLAY").then(|| OsString::from("wayland-1"));
        assert!(can_show_dialog(var));
        assert!(!can_show_dialog(|_| None));
        assert!(!can_show_dialog(|_| Some(OsString::new())));
    }
    #[test]
    fn open_configured_store_honors_the_env_override() {
        let dir = tempfile::tempdir().unwrap();
        let chosen = dir.path().join("chosen.db");
        let store = open_configured_store(Some(chosen.as_os_str()), dir.path(), false).unwrap();
        assert!(store.list_cards().unwrap().is_empty());
        assert!(chosen.exists());
        assert!(!dir.path().join(db_path::RELEASE_DIR).exists());
        let broken = dir.path().join("broken.db");
        std::fs::write(&broken, b"this is not a sqlite database at all, just text").unwrap();
        let Err(message) = open_configured_store(Some(broken.as_os_str()), dir.path(), false)
        else {
            panic!("a broken file must not open")
        };
        assert!(message.contains("broken.db"));
    }
    #[test]
    fn open_store_reports_the_path_and_cause() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("broken.db");
        std::fs::write(&path, b"this is not a sqlite database at all, just text").unwrap();
        let Err(message) = open_store(&path) else {
            panic!("a broken file must not open")
        };
        assert!(message.contains("broken.db"));
        assert!(message.contains("not a database"));
    }
}
