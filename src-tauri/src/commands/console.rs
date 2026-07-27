use serde::Serialize;
use tauri::State;

use crate::commands::keys::ConnectionManager;
use crate::redis::types::RedisValue;

#[derive(Serialize)]
pub struct CommandResult {
    pub result: RedisValue,
    pub duration_ms: u64,
}

/// Parse a Redis command string, respecting single and double quotes.
/// e.g. `SET key "hello world"` → ["SET", "key", "hello world"]
fn parse_command(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    while let Some(&ch) = chars.peek() {
        match ch {
            '"' | '\'' => {
                let quote = ch;
                chars.next(); // consume opening quote
                while let Some(&c) = chars.peek() {
                    if c == quote {
                        chars.next(); // consume closing quote
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        if let Some(&escaped) = chars.peek() {
                            current.push(escaped);
                            chars.next();
                        }
                    } else {
                        current.push(c);
                        chars.next();
                    }
                }
            }
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            _ => {
                current.push(ch);
                chars.next();
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[tauri::command]
pub async fn execute_command(
    connection_id: String,
    command: String,
    manager: State<'_, ConnectionManager>,
) -> Result<CommandResult, String> {
    let client = {
        let map = manager.lock().await;
        std::sync::Arc::clone(map
            .get(&connection_id)
            .ok_or_else(|| format!("connection '{}' not found", connection_id))?)
    };
    let parts = parse_command(&command);
    if parts.is_empty() {
        return Err("Empty command".to_string());
    }
    let cmd = &parts[0];
    let args: Vec<String> = parts[1..].to_vec();
    let start = std::time::Instant::now();
    let result = client.execute(cmd, args).await.map_err(|e| e.to_string())?;
    let duration_ms = start.elapsed().as_millis() as u64;
    Ok(CommandResult { result, duration_ms })
}
