pub mod commands;
pub mod details;
pub mod domain;
pub mod mock_data;
pub mod store;
#[cfg(test)]
mod store_contract;

use commands::AppState;
use store::MockStore;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            store: Box::new(MockStore::new(mock_data::sample_cards())),
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_board,
            commands::move_card,
            commands::update_card,
            commands::window_controls
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
