mod commands;
mod state;
mod storage;
mod sync;

use commands::*;
use state::AppState;
use sync::SyncState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState::new())
        .manage(SyncState::new())
        .invoke_handler(tauri::generate_handler![
            // Vault status
            get_vault_status,
            // Vault operations
            create_vault,
            unlock_vault,
            lock_vault,
            wipe_vault,
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
            // Sync
            get_sync_status,
            enable_sync,
            disable_sync,
            trigger_sync,
            check_remote_commands,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
