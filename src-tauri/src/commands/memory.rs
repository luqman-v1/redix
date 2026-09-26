use serde::Serialize;
use std::sync::Arc;
use tauri::State;

use super::keys::ConnectionManager;

#[derive(Serialize)]
pub struct MemoryKeyInfo {
    pub key: String,
    pub bytes: u64,
    pub key_type: String,
}

#[derive(Serialize)]
pub struct MemoryAnalysisResult {
    pub total_keys_scanned: usize,
    pub top_keys: Vec<MemoryKeyInfo>,
}

#[tauri::command]
pub async fn analyze_memory(
    connection_id: String,
    sample_size: usize,
    manager: State<'_, ConnectionManager>,
) -> Result<MemoryAnalysisResult, String> {
    const MAX_SAMPLE_SIZE: usize = 50_000;
    const SCAN_COUNT: u64 = 1000;
    const CHUNK_SIZE: usize = 200;
    let sample_size = sample_size.clamp(1, MAX_SAMPLE_SIZE);
    let client = {
        let map = manager.lock().await;
        Arc::clone(
            map.get(&connection_id)
                .ok_or_else(|| format!("connection '{}' not found", connection_id))?,
        )
    };

    let mut scanned_keys = Vec::new();
    let mut cursor = 0;

    // Scan up to sample_size
    loop {
        let (next_cursor, keys) = client.scan_keys(cursor, SCAN_COUNT, None).await?;
        for k in keys {
            scanned_keys.push(k);
        }
        cursor = next_cursor;
        if cursor == 0 || scanned_keys.len() >= sample_size {
            break;
        }
    }

    if scanned_keys.len() > sample_size {
        scanned_keys.truncate(sample_size);
    }

    let mut results = Vec::with_capacity(scanned_keys.len());

    // Process in chunks to avoid overwhelming the connection
    for chunk in scanned_keys.chunks(CHUNK_SIZE) {
        let mut cmds = Vec::with_capacity(chunk.len() * 2);
        for key in chunk {
            cmds.push(("MEMORY".to_string(), vec!["USAGE".to_string(), key.clone()]));
            cmds.push(("TYPE".to_string(), vec![key.clone()]));
        }

        match client.execute_pipeline(cmds).await {
            Ok(values) => {
                let mut i = 0;
                for key in chunk {
                    let bytes = match values.get(i) {
                        Some(crate::redis::types::RedisValue::Integer(b)) => *b as u64,
                        _ => 0,
                    };
                    i += 1;

                    let key_type = match values.get(i) {
                        Some(crate::redis::types::RedisValue::Status(s)) => s.clone(),
                        Some(crate::redis::types::RedisValue::String(s)) => s.clone(),
                        _ => "unknown".to_string(),
                    };
                    i += 1;

                    results.push(MemoryKeyInfo {
                        key: key.clone(),
                        bytes,
                        key_type,
                    });
                }
            }
            Err(e) => {
                // If pipeline fails for a chunk, skip this chunk
                log::error!("Pipeline error analyzing memory: {}", e);
            }
        }
    }

    // Sort by bytes descending
    results.sort_by(|a, b| b.bytes.cmp(&a.bytes));

    // Take top 1000
    let top_keys = results.into_iter().take(1000).collect();

    Ok(MemoryAnalysisResult {
        total_keys_scanned: scanned_keys.len(),
        top_keys,
    })
}
