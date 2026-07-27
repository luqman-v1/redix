use std::sync::Arc;
use serde::Serialize;
use tauri::State;
use crate::redis::types::RedisValue;

use super::keys::ConnectionManager;

#[derive(Serialize)]
pub struct SlowLogEntry {
    pub id: i64,
    pub timestamp: i64,
    pub duration_micros: i64,
    pub command: String,
    pub client_address: String,
    pub client_name: String,
}

#[tauri::command]
pub async fn get_slow_logs(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<SlowLogEntry>, String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };

    let res = client
        .execute("SLOWLOG", vec!["GET".to_string(), "200".to_string()])
        .await?;

    let mut logs = Vec::new();

    if let RedisValue::Array(entries) = res {
        for entry in entries {
            if let RedisValue::Array(fields) = entry {
                // Minimum 4 fields in SLOWLOG GET output
                if fields.len() < 4 {
                    continue;
                }

                let id = match &fields[0] {
                    RedisValue::Integer(v) => *v,
                    _ => 0,
                };

                let timestamp = match &fields[1] {
                    RedisValue::Integer(v) => *v,
                    _ => 0,
                };

                let duration_micros = match &fields[2] {
                    RedisValue::Integer(v) => *v,
                    _ => 0,
                };

                // Command is an array of strings
                let command = match &fields[3] {
                    RedisValue::Array(cmd_args) => {
                        let mut args = Vec::new();
                        for arg in cmd_args {
                            match arg {
                                RedisValue::String(s) => args.push(s.clone()),
                                RedisValue::Integer(i) => args.push(i.to_string()),
                                _ => args.push("?".to_string()),
                            }
                        }
                        args.join(" ")
                    }
                    _ => "".to_string(),
                };

                // Field 4 and 5 are client address and name (Redis 4.0+)
                let client_address = if fields.len() > 4 {
                    match &fields[4] {
                        RedisValue::String(s) => s.clone(),
                        _ => "".to_string(),
                    }
                } else {
                    "".to_string()
                };

                let client_name = if fields.len() > 5 {
                    match &fields[5] {
                        RedisValue::String(s) => s.clone(),
                        _ => "".to_string(),
                    }
                } else {
                    "".to_string()
                };

                logs.push(SlowLogEntry {
                    id,
                    timestamp,
                    duration_micros,
                    command,
                    client_address,
                    client_name,
                });
            }
        }
    }

    Ok(logs)
}

#[tauri::command]
pub async fn reset_slow_logs(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };

    client
        .execute("SLOWLOG", vec!["RESET".to_string()])
        .await?;
    
    Ok(())
}
