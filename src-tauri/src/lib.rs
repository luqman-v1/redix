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
        .manage(Arc::new(Mutex::new(HashMap::new())) as commands::history::HistoryStore)
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
            commands::values::get_string_value,
            commands::values::set_string_value,
            commands::values::get_hash_all,
            commands::values::set_hash_field,
            commands::values::del_hash_field,
            commands::values::get_list_range,
            commands::values::set_list_value,
            commands::values::list_push,
            commands::values::list_pop,
            commands::values::get_set_members,
            commands::values::add_set_member,
            commands::values::del_set_member,
            commands::values::get_sorted_set_range,
            commands::values::add_sorted_set,
            commands::values::del_sorted_set_member,
            commands::values::get_stream_range,
            commands::values::get_hyperloglog_count,
            commands::values::get_geo_members,
            commands::console::execute_command,
            commands::history::get_history,
            commands::history::add_to_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
