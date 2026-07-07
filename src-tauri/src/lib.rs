pub mod commands;
pub mod config;
pub mod redis;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(config::ConnectionStore::default())
        .manage(Arc::new(Mutex::new(HashMap::new())) as commands::keys::ConnectionManager)
        .invoke_handler(tauri::generate_handler![
            commands::connections::get_connections,
            commands::connections::add_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::test_connection,
            commands::keys::scan_keys,
            commands::keys::get_key_type,
            commands::keys::get_key_ttl,
            commands::keys::delete_key,
            commands::keys::rename_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
