use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tokio::sync::Mutex;

use crate::redis::client::RedisClient;

pub type ConnectionManager = Arc<Mutex<HashMap<String, Arc<dyn RedisClient>>>>;

#[derive(Serialize)]
pub struct KeyInfo {
    pub key: String,
    pub ttl: i64,
}

#[derive(Serialize)]
pub struct KeyMeta {
    #[serde(rename = "type")]
    pub key_type: String,
    pub ttl: i64,
}

#[derive(Serialize)]
pub struct ScanResult {
    pub cursor: u64,
    pub keys: Vec<KeyInfo>,
}

async fn ttls_for_keys(
    client: &std::sync::Arc<dyn crate::redis::client::RedisClient>,
    keys: &[String],
) -> Vec<i64> {
    if keys.is_empty() {
        return Vec::new();
    }
    let cmds: Vec<(String, Vec<String>)> = keys
        .iter()
        .map(|k| ("TTL".to_string(), vec![k.clone()]))
        .collect();
    if let Ok(values) = client.execute_pipeline(cmds).await {
        if values.len() == keys.len() {
            return values
                .into_iter()
                .map(|v| match v {
                    crate::redis::types::RedisValue::Integer(n) => n,
                    _ => -1,
                })
                .collect();
        }
    }
    let futures = keys.iter().map(|k| client.get_ttl(k));
    futures_util::future::join_all(futures)
        .await
        .into_iter()
        .map(|r| r.unwrap_or(-1))
        .collect()
}

#[tauri::command]
pub async fn scan_keys(
    connection_id: String,
    cursor: u64,
    count: u64,
    pattern: Option<String>,
    manager: State<'_, ConnectionManager>,
) -> Result<ScanResult, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    let (next_cursor, keys) = client.scan_keys(cursor, count, pattern.as_deref()).await?;
    let ttls = ttls_for_keys(&client, &keys).await;
    let key_infos = keys
        .into_iter()
        .zip(ttls)
        .map(|(key, ttl)| KeyInfo { key, ttl })
        .collect();

    Ok(ScanResult {
        cursor: next_cursor,
        keys: key_infos,
    })
}

#[tauri::command]
pub async fn get_key_type(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<String, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    client.get_type(&key).await
}

#[tauri::command]
pub async fn get_key_meta(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<KeyMeta, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    let cmds = vec![
        ("TYPE".to_string(), vec![key.clone()]),
        ("TTL".to_string(), vec![key.clone()]),
    ];
    if let Ok(values) = client.execute_pipeline(cmds).await {
        if values.len() == 2 {
            let key_type = match &values[0] {
                crate::redis::types::RedisValue::Status(s)
                | crate::redis::types::RedisValue::String(s) => s.clone(),
                _ => "none".to_string(),
            };
            let ttl = match &values[1] {
                crate::redis::types::RedisValue::Integer(n) => *n,
                _ => -2,
            };
            return Ok(KeyMeta { key_type, ttl });
        }
    }
    Ok(KeyMeta {
        key_type: client.get_type(&key).await?,
        ttl: client.get_ttl(&key).await?,
    })
}

#[tauri::command]
pub async fn get_key_ttl(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    client.get_ttl(&key).await
}

#[tauri::command]
pub async fn delete_key(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    client.del(vec![&key]).await
}

#[tauri::command]
pub async fn rename_key(
    connection_id: String,
    old_name: String,
    new_name: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };
    client.rename(&old_name, &new_name).await
}

#[tauri::command]
pub async fn set_key_ttl(
    connection_id: String,
    key: String,
    ttl: i64,
    manager: tauri::State<'_, ConnectionManager>,
) -> Result<(), String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };

    if ttl <= 0 {
        client.persist(&key).await.map(|_| ())
    } else {
        client.set_ttl(&key, ttl as u64).await.map(|_| ())
    }
}
