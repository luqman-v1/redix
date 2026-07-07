use tauri::State;

use crate::config::{ConnectionConfig, ConnectionStore};
use crate::commands::keys::ConnectionManager;
use crate::redis::standalone::StandaloneClient;
use crate::redis::client::RedisClient;

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
pub async fn connect_to_server(
    connection_id: String,
    store: State<'_, ConnectionStore>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let configs = store.load()?;
    let config = configs
        .into_iter()
        .find(|c| c.id.to_string() == connection_id)
        .ok_or("Connection config not found")?;
    let mut client = StandaloneClient::new(config);
    client.connect().await.map_err(|e| format!("Connect failed: {e}"))?;
    let mut map = manager.lock().await;
    map.insert(connection_id, Box::new(client) as Box<dyn RedisClient>);
    Ok(())
}

#[tauri::command]
pub async fn disconnect_server(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let mut map = manager.lock().await;
    if let Some(mut client) = map.remove(&connection_id) {
        client.disconnect().await.map_err(|e| format!("Disconnect failed: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
pub fn test_connection(config: ConnectionConfig) -> Result<bool, String> {
    if config.host.is_empty() {
        return Err("host must not be empty".into());
    }
    if config.port == 0 {
        return Err("port must be greater than 0".into());
    }
    Ok(true)
}

#[tauri::command]
pub async fn reconnect(
    connection_id: String,
    store: State<'_, ConnectionStore>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let configs = store.load()?;
    let config = configs
        .into_iter()
        .find(|c| c.id.to_string() == connection_id)
        .ok_or("config not found")?;
    let mut client = StandaloneClient::new(config);
    client.connect().await?;
    let mut map = manager.lock().await;
    map.insert(connection_id, Box::new(client) as Box<dyn RedisClient>);
    Ok(())
}
