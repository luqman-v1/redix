use log::{error, info};
use std::sync::Arc;
use tauri::State;

use crate::commands::keys::ConnectionManager;
use crate::config::{ConnectionConfig, ConnectionStore};
use crate::redis::client::create_client;

async fn load_config(
    store: &State<'_, ConnectionStore>,
    connection_id: &str,
) -> Result<ConnectionConfig, String> {
    store
        .load()?
        .into_iter()
        .find(|c| c.id.to_string() == connection_id)
        .ok_or_else(|| "Connection config not found".to_string())
}

async fn insert_client(
    manager: &State<'_, ConnectionManager>,
    connection_id: &str,
    client: Box<dyn crate::redis::client::RedisClient>,
) {
    let mut map = manager.lock().await;
    let old_client = map.remove(connection_id);
    drop(map);
    if let Some(mut old_client) = old_client {
        if let Some(c) = Arc::get_mut(&mut old_client) {
            let _ = c.disconnect().await;
        }
    }
    let mut map = manager.lock().await;
    let logged_client = crate::redis::LoggingClient { inner: client };
    map.insert(connection_id.to_string(), Arc::new(logged_client));
}

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
    let config = load_config(&store, &connection_id).await.map_err(|e| {
        error!(
            "[connect_to_server] config not found for id={}: {}",
            connection_id, e
        );
        e
    })?;
    let mut client = create_client(config);
    client.connect().await.map_err(|e| {
        error!("[connect_to_server] connect failed: {}", e);
        format!("Connect failed: {e}")
    })?;
    insert_client(&manager, &connection_id, client).await;
    info!("[connect_to_server] connected to {}", connection_id);
    Ok(())
}

#[tauri::command]
pub async fn disconnect_server(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    info!("[disconnect_server] connection_id={}", connection_id);
    let client = {
        let mut map = manager.lock().await;
        map.remove(&connection_id)
    };

    if let Some(mut client) = client {
        if let Some(c) = Arc::get_mut(&mut client) {
            c.disconnect().await.map_err(|e| {
                error!("[disconnect_server] {}", e);
                format!("Disconnect failed: {e}")
            })?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<bool, String> {
    if config.host.is_empty() {
        return Err("host must not be empty".into());
    }
    if config.port == 0 {
        return Err("port must be greater than 0".into());
    }
    let mut client = create_client(config);
    match client.connect().await {
        Ok(_) => {
            let _ = client.disconnect().await;
            Ok(true)
        }
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

#[tauri::command]
pub async fn reconnect(
    connection_id: String,
    store: State<'_, ConnectionStore>,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    info!("[reconnect] connection_id={}", connection_id);
    let config = load_config(&store, &connection_id).await.map_err(|e| {
        error!(
            "[reconnect] config not found for id={}: {}",
            connection_id, e
        );
        e
    })?;
    let mut client = create_client(config);
    client.connect().await.map_err(|e| {
        error!("[reconnect] connect failed: {}", e);
        e
    })?;
    insert_client(&manager, &connection_id, client).await;
    Ok(())
}

#[tauri::command]
pub async fn get_server_info(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| "Not connected".to_string())?,
        )
    };
    let info_val = client.execute("INFO", vec![]).await?;
    let info_str = match info_val {
        crate::redis::types::RedisValue::String(s) => s,
        crate::redis::types::RedisValue::Status(s) => s,
        other => {
            log::error!("[get_server_info] Invalid INFO response: {:?}", other);
            return Err(format!("Invalid INFO response: {:?}", other));
        }
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
