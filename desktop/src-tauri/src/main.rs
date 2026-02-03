#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;
mod state;
mod storage;

use commands::*;
use state::AppState;

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Vault status
            get_vault_status,
            // Vault operations
            create_vault,
            unlock_vault,
            lock_vault,
            // Item operations
            get_all_items,
            get_item,
            add_item,
            update_item,
            delete_item,
            search_items,
            get_favorites,
            // Password generation
            generate_password_cmd,
            generate_passphrase_cmd,
            // Settings
            get_auto_lock_timeout,
            set_auto_lock_timeout,
            check_auto_lock,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
