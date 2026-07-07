use tauri::State;

use crate::config::{ConnectionConfig, ConnectionStore};

#[tauri::command]
pub fn get_connections(store: State<'_, ConnectionStore>) -> Result<Vec<ConnectionConfig>, String> {
    store.load()
}

#[tauri::command]
pub fn add_connection(
    store: State<'_, ConnectionStore>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    store.add(config.clone())?;
    Ok(config)
}

#[tauri::command]
pub fn update_connection(
    store: State<'_, ConnectionStore>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    store.update(config.clone())?;
    Ok(config)
}

#[tauri::command]
pub fn delete_connection(store: State<'_, ConnectionStore>, id: String) -> Result<(), String> {
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| format!("invalid uuid: {}", e))?;
    store.delete(uuid)
}

#[tauri::command]
pub fn test_connection(config: ConnectionConfig) -> Result<bool, String> {
    if config.host.is_empty() {
        return Err("host must not be empty".into());
    }
    if config.port == 0 {
        return Err("port must be greater than 0".into());
    }
    // ponytail: actual TCP/Redis ping skipped, add when connection pool impl lands
    Ok(true)
}
