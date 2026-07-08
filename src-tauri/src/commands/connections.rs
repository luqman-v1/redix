use log::{error, info};
use tauri::State;

use crate::config::{ConnectionConfig, ConnectionStore};
use crate::commands::keys::ConnectionManager;
use crate::redis::standalone::StandaloneClient;
use crate::redis::client::RedisClient;

#[tauri::command]
pub fn get_connections(store: State<'_, ConnectionStore>) -> Result<Vec<ConnectionConfig>, String> {
    store.load().map_err(|e| {
        error!("[get_connections] {}", e);
        e
    })
}

#[tauri::command]
pub fn add_connection(
    store: State<'_, ConnectionStore>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    store.add(config.clone()).map_err(|e| {
        error!("[add_connection] {}", e);
        e
    })?;
    Ok(config)
}

#[tauri::command]
pub fn update_connection(
    store: State<'_, ConnectionStore>,
    config: ConnectionConfig,
) -> Result<ConnectionConfig, String> {
    store.update(config.clone()).map_err(|e| {
        error!("[update_connection] {}", e);
        e
    })?;
    Ok(config)
}

#[tauri::command]
pub fn delete_connection(store: State<'_, ConnectionStore>, id: String) -> Result<(), String> {
    info!("[delete_connection] id={}", id);
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| {
        error!("[delete_connection] invalid uuid '{}': {}", id, e);
        format!("invalid uuid: {}", e)
    })?;
    store.delete(uuid).map_err(|e| {
        error!("[delete_connection] {}", e);
        e
    })
}

#[tauri::command]
pub async fn connect_to_server(
    connection_id: String,
    store: State<'_, ConnectionStore>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    info!("[connect_to_server] connection_id={}", connection_id);
    let configs = store.load()?;
    let config = configs
        .into_iter()
        .find(|c| c.id.to_string() == connection_id)
        .ok_or_else(|| {
            error!("[connect_to_server] config not found for id={}", connection_id);
            "Connection config not found".to_string()
        })?;
    let mut client = StandaloneClient::new(config);
    client.connect().await.map_err(|e| {
        error!("[connect_to_server] connect failed: {}", e);
        format!("Connect failed: {e}")
    })?;
    let mut map = manager.lock().await;
    let logged_client = crate::redis::LoggingClient { inner: Box::new(client) };
    map.insert(connection_id.clone(), Box::new(logged_client) as Box<dyn RedisClient>);
    info!("[connect_to_server] connected to {}", connection_id);
    Ok(())
}

#[tauri::command]
pub async fn disconnect_server(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    info!("[disconnect_server] connection_id={}", connection_id);
    let mut map = manager.lock().await;
    if let Some(mut client) = map.remove(&connection_id) {
        client.disconnect().await.map_err(|e| {
            error!("[disconnect_server] {}", e);
            format!("Disconnect failed: {e}")
        })?;
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
    info!("[reconnect] connection_id={}", connection_id);
    let configs = store.load()?;
    let config = configs
        .into_iter()
        .find(|c| c.id.to_string() == connection_id)
        .ok_or_else(|| {
            error!("[reconnect] config not found for id={}", connection_id);
            "config not found".to_string()
        })?;
    let mut client = StandaloneClient::new(config);
    client.connect().await.map_err(|e| {
        error!("[reconnect] connect failed: {}", e);
        e
    })?;
    let mut map = manager.lock().await;
    let logged_client = crate::redis::LoggingClient { inner: Box::new(client) };
    map.insert(connection_id, Box::new(logged_client) as Box<dyn RedisClient>);
    Ok(())
}

#[tauri::command]
pub async fn get_server_info(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let mut map = manager.lock().await;
    let client = map.get_mut(&connection_id).ok_or_else(|| "Not connected".to_string())?;
    
    let info_val = client.execute("INFO", vec![]).await?;
    let info_str = match info_val {
        crate::redis::client::RedisValue::String(s) => s,
        crate::redis::client::RedisValue::Status(s) => s,
        _ => return Err("Invalid INFO response".into()),
    };

    let mut result = std::collections::HashMap::new();
    for line in info_str.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            result.insert(k.to_string(), v.to_string());
        }
    }
    
    Ok(result)
}
