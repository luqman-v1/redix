use serde::Serialize;
use tauri::State;

use crate::commands::keys::ConnectionManager;
use crate::redis::client::RedisValue;

#[derive(Serialize)]
pub struct CommandResult {
    pub result: RedisValue,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn execute_command(
    connection_id: String,
    command: String,
    manager: State<'_, ConnectionManager>,
) -> Result<CommandResult, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let cmd = parts[0];
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    let start = std::time::Instant::now();
    let result = client.execute(cmd, args).await.map_err(|e| e.to_string())?;
    let duration_ms = start.elapsed().as_millis() as u64;
    Ok(CommandResult { result, duration_ms })
}
