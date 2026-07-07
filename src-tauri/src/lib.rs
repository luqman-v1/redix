pub mod commands;
pub mod config;
pub mod redis;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(config::ConnectionStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::connections::get_connections,
            commands::connections::add_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::test_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
