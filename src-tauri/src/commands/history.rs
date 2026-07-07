use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

pub type HistoryStore = Arc<Mutex<HashMap<String, Vec<String>>>>;

#[tauri::command]
pub async fn get_history(
    connection_id: String,
    store: State<'_, HistoryStore>,
) -> Result<Vec<String>, String> {
    let map = store.lock().await;
    Ok(map.get(&connection_id).cloned().unwrap_or_default())
}

#[tauri::command]
pub async fn add_to_history(
    connection_id: String,
    command: String,
    store: State<'_, HistoryStore>,
) -> Result<(), String> {
    let mut map = store.lock().await;
    let list = map.entry(connection_id).or_insert_with(Vec::new);
    list.push(command);
    if list.len() > 1000 {
        list.remove(0);
    }
    Ok(())
}
