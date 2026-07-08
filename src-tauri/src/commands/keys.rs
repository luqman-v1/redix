use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::redis::client::RedisClient;

pub type ConnectionManager = Arc<Mutex<HashMap<String, Box<dyn RedisClient>>>>;

#[derive(Serialize)]
pub struct ScanResult {
    pub cursor: u64,
    pub keys: Vec<String>,
}

#[tauri::command]
pub async fn scan_keys(
    connection_id: String,
    cursor: u64,
    count: u64,
    pattern: Option<String>,
    manager: State<'_, ConnectionManager>,
) -> Result<ScanResult, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let (next_cursor, keys) = client.scan_keys(cursor, count, pattern.as_deref()).await?;
    Ok(ScanResult {
        cursor: next_cursor,
        keys,
    })
}

#[tauri::command]
pub async fn get_key_type(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<String, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    client.get_type(&key).await
}

#[tauri::command]
pub async fn get_key_ttl(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    client.get_ttl(&key).await
}

#[tauri::command]
pub async fn delete_key(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    client.del(vec![&key]).await
}

#[tauri::command]
pub async fn rename_key(
    connection_id: String,
    old_name: String,
    new_name: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    client.rename(&old_name, &new_name).await
}

#[tauri::command]
pub async fn set_key_ttl(
    connection_id: String,
    key: String,
    ttl: i64,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    
    if ttl <= 0 {
        client.persist(&key).await.map(|_| ())
    } else {
        client.set_ttl(&key, ttl as u64).await.map(|_| ())
    }
}
