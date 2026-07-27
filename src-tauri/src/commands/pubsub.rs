use tauri::{AppHandle, Emitter};
use futures_util::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use std::collections::HashMap;

use crate::config::connection::ConnectionConfig;
use crate::redis::tls::build_tls_certificates;

// A global store for active pubsub tasks so we can cancel them
pub type PubSubTasks = Arc<Mutex<HashMap<String, tokio::task::JoinHandle<()>>>>;

#[derive(Clone, serde::Serialize)]
struct PubSubMessage {
    channel: String,
    payload: String,
}

#[tauri::command]
pub async fn subscribe_channel(
    app: AppHandle,
    config: ConnectionConfig,
    channel: String,
    tasks: tauri::State<'_, PubSubTasks>,
) -> Result<String, String> {
    let sub_id = Uuid::new_v4().to_string();
    let app_clone = app.clone();
    let event_name = format!("pubsub-{}", sub_id);

    let use_ssl = config.use_ssl || config.ssl.is_some();
    let scheme = if use_ssl { "rediss://" } else { "redis://" };
    let mut url = String::from(scheme);
    match (&config.username, &config.password) {
        (Some(u), Some(p)) => url.push_str(&format!("{}:{}@", u, p)),
        (None, Some(p)) => url.push_str(&format!(":{}@", p)),
        _ => {}
    }
    url.push_str(&format!("{}:{}", config.host, config.port));
    if let Some(ssl) = &config.ssl {
        if ssl.skip_verify {
            url.push_str("#insecure");
        }
    }

    use crate::config::connection::ConnectionType;
    use redis::IntoConnectionInfo;
    use redis::sentinel::Sentinel;

    let client = if config.connection_type == ConnectionType::Sentinel {
        let info = url.into_connection_info().map_err(|e| e.to_string())?;
        let mut sentinel = Sentinel::build(vec![info]).map_err(|e| e.to_string())?;
        let master = config.sentinel_master_name.as_deref().unwrap_or("mymaster");
        sentinel.async_master_for(master, None).await.map_err(|e| e.to_string())?
    } else if use_ssl {
        let certs = build_tls_certificates(&config)?;
        redis::Client::build_with_tls(url, certs).map_err(|e| e.to_string())?
    } else {
        redis::Client::open(url).map_err(|e| e.to_string())?
    };

    let (tx, rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn(async move {
        // We try to get pubsub connection
        match client.get_async_pubsub().await {
            Ok(mut pubsub) => {
                if let Ok(_) = pubsub.subscribe(&channel).await {
                    let _ = tx.send(Ok(()));
                    let mut stream = pubsub.on_message();
                    while let Some(msg) = stream.next().await {
                        if let Ok(payload) = msg.get_payload::<String>() {
                            let _ = app_clone.emit(&event_name, PubSubMessage {
                                channel: msg.get_channel_name().to_string(),
                                payload,
                            });
                        }
                    }
                } else {
                    let _ = tx.send(Err("Failed to subscribe".to_string()));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e.to_string()));
            }
        }
    });

    match rx.await {
        Ok(Ok(_)) => {
            tasks.lock().await.insert(sub_id.clone(), handle);
            Ok(sub_id)
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("PubSub task panicked before subscribing".to_string()),
    }
}

#[tauri::command]
pub async fn unsubscribe_channel(
    sub_id: String,
    tasks: tauri::State<'_, PubSubTasks>,
) -> Result<(), String> {
    if let Some(handle) = tasks.lock().await.remove(&sub_id) {
        handle.abort();
    }
    Ok(())
}
