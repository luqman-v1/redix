use serde::Serialize;
use tauri::State;

use super::keys::ConnectionManager;
use crate::redis::client::RedisValue;

#[derive(Serialize)]
pub struct StreamEntry {
    pub id: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Serialize)]
pub struct GeoMember {
    pub member: String,
    pub longitude: f64,
    pub latitude: f64,
    pub score: f64,
}

fn parse_string(val: RedisValue) -> Result<String, String> {
    match val {
        RedisValue::String(s) | RedisValue::Status(s) => Ok(s),
        RedisValue::Nil => Err("key does not exist".into()),
        RedisValue::Error(e) => Err(e),
        other => Ok(other.to_display_string()),
    }
}

fn parse_optional_string(val: RedisValue) -> Option<String> {
    match val {
        RedisValue::Nil => None,
        RedisValue::String(s) | RedisValue::Status(s) => Some(s),
        other => Some(other.to_display_string()),
    }
}

fn parse_i64(val: RedisValue) -> Result<i64, String> {
    match val {
        RedisValue::Integer(n) => Ok(n),
        RedisValue::Error(e) => Err(e),
        _ => Err("unexpected response type".into()),
    }
}

fn parse_ok(val: RedisValue) -> Result<(), String> {
    match val {
        RedisValue::Status(s) if s == "OK" => Ok(()),
        RedisValue::Status(_) => Ok(()),
        RedisValue::Integer(_) => Ok(()),
        RedisValue::Error(e) => Err(e),
        _ => Ok(()),
    }
}

fn parse_string_array(val: RedisValue) -> Result<Vec<String>, String> {
    match val {
        RedisValue::Array(arr) => arr
            .into_iter()
            .map(|v| match v {
                RedisValue::String(s) | RedisValue::Status(s) => Ok(s),
                RedisValue::Integer(n) => Ok(n.to_string()),
                RedisValue::Float(f) => Ok(f.to_string()),
                RedisValue::Nil => Ok(String::new()),
                RedisValue::Error(e) => Err(e),
                other => Ok(other.to_display_string()),
            })
            .collect(),
        RedisValue::Nil => Ok(vec![]),
        RedisValue::Error(e) => Err(e),
        _ => Err("unexpected response type".into()),
    }
}

// --- String commands ---

#[tauri::command]
pub async fn get_string_value(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<String, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("GET", vec![key]).await?;
    parse_string(val)
}

#[tauri::command]
pub async fn set_string_value(
    connection_id: String,
    key: String,
    value: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("SET", vec![key, value]).await?;
    parse_ok(val)
}

// --- Hash commands ---

#[tauri::command]
pub async fn get_hash_all(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<(String, String)>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("HGETALL", vec![key]).await?;
    match val {
        RedisValue::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len() / 2);
            let mut iter = arr.into_iter();
            while let Some(field) = iter.next() {
                let value = iter.next().unwrap_or(RedisValue::Nil);
                let f = match field {
                    RedisValue::String(s) | RedisValue::Status(s) => s,
                    other => other.to_display_string(),
                };
                let v = match value {
                    RedisValue::String(s) | RedisValue::Status(s) => s,
                    other => other.to_display_string(),
                };
                result.push((f, v));
            }
            Ok(result)
        }
        RedisValue::Nil => Ok(vec![]),
        RedisValue::Error(e) => Err(e),
        _ => Err("unexpected response type".into()),
    }
}

#[tauri::command]
pub async fn set_hash_field(
    connection_id: String,
    key: String,
    field: String,
    value: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("HSET", vec![key, field, value]).await?;
    parse_ok(val)
}

#[tauri::command]
pub async fn del_hash_field(
    connection_id: String,
    key: String,
    field: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("HDEL", vec![key, field]).await?;
    parse_ok(val)
}

// --- List commands ---

#[tauri::command]
pub async fn get_list_range(
    connection_id: String,
    key: String,
    start: i64,
    stop: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<String>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client
        .execute("LRANGE", vec![key, start.to_string(), stop.to_string()])
        .await?;
    parse_string_array(val)
}

#[tauri::command]
pub async fn set_list_value(
    connection_id: String,
    key: String,
    index: i64,
    value: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client
        .execute("LSET", vec![key, index.to_string(), value])
        .await?;
    parse_ok(val)
}

#[tauri::command]
pub async fn list_push(
    connection_id: String,
    key: String,
    value: String,
    side: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let cmd = match side.as_str() {
        "left" => "LPUSH",
        "right" => "RPUSH",
        _ => return Err("side must be 'left' or 'right'".into()),
    };
    let val = client.execute(cmd, vec![key, value]).await?;
    parse_ok(val)
}

#[tauri::command]
pub async fn list_pop(
    connection_id: String,
    key: String,
    side: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Option<String>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let cmd = match side.as_str() {
        "left" => "LPOP",
        "right" => "RPOP",
        _ => return Err("side must be 'left' or 'right'".into()),
    };
    let val = client.execute(cmd, vec![key]).await?;
    Ok(parse_optional_string(val))
}

// --- Set commands ---

#[tauri::command]
pub async fn get_set_members(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<String>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("SMEMBERS", vec![key]).await?;
    parse_string_array(val)
}

#[tauri::command]
pub async fn add_set_member(
    connection_id: String,
    key: String,
    member: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("SADD", vec![key, member]).await?;
    parse_ok(val)
}

#[tauri::command]
pub async fn del_set_member(
    connection_id: String,
    key: String,
    member: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("SREM", vec![key, member]).await?;
    parse_ok(val)
}

// --- Sorted set commands ---

#[tauri::command]
pub async fn get_sorted_set_range(
    connection_id: String,
    key: String,
    start: i64,
    stop: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<(String, f64)>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client
        .execute(
            "ZRANGE",
            vec![key, start.to_string(), stop.to_string(), "WITHSCORES".into()],
        )
        .await?;
    match val {
        RedisValue::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len() / 2);
            let mut iter = arr.into_iter();
            while let Some(member) = iter.next() {
                let score = iter.next().unwrap_or(RedisValue::Nil);
                let m = match member {
                    RedisValue::String(s) | RedisValue::Status(s) => s,
                    other => other.to_display_string(),
                };
                let s = match score {
                    RedisValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                    RedisValue::Float(f) => f,
                    RedisValue::Integer(n) => n as f64,
                    _ => 0.0,
                };
                result.push((m, s));
            }
            Ok(result)
        }
        RedisValue::Nil => Ok(vec![]),
        RedisValue::Error(e) => Err(e),
        _ => Err("unexpected response type".into()),
    }
}

#[tauri::command]
pub async fn add_sorted_set(
    connection_id: String,
    key: String,
    score: f64,
    member: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client
        .execute("ZADD", vec![key, score.to_string(), member])
        .await?;
    parse_ok(val)
}

#[tauri::command]
pub async fn del_sorted_set_member(
    connection_id: String,
    key: String,
    member: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("ZREM", vec![key, member]).await?;
    parse_ok(val)
}

// --- Stream commands ---

#[tauri::command]
pub async fn get_stream_range(
    connection_id: String,
    key: String,
    start: String,
    end: String,
    count: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<StreamEntry>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let args = if count > 0 {
        vec![
            key,
            start,
            end,
            "COUNT".into(),
            count.to_string(),
        ]
    } else {
        vec![key, start, end]
    };
    let val = client.execute("XRANGE", args).await?;
    parse_stream_entries(val)
}

fn parse_stream_entries(val: RedisValue) -> Result<Vec<StreamEntry>, String> {
    match val {
        RedisValue::Array(entries) => {
            let mut result = Vec::with_capacity(entries.len());
            for entry in entries {
                if let RedisValue::Array(pair) = entry {
                    if pair.len() >= 2 {
                        let id = match &pair[0] {
                            RedisValue::String(s) | RedisValue::Status(s) => s.clone(),
                            other => other.to_display_string(),
                        };
                        let fields = parse_field_pairs(&pair[1]);
                        result.push(StreamEntry { id, fields });
                    }
                }
            }
            Ok(result)
        }
        RedisValue::Nil => Ok(vec![]),
        RedisValue::Error(e) => Err(e),
        _ => Err("unexpected response type".into()),
    }
}

fn parse_field_pairs(val: &RedisValue) -> Vec<(String, String)> {
    match val {
        RedisValue::Array(fields) => {
            let mut result = Vec::with_capacity(fields.len() / 2);
            let mut iter = fields.iter();
            while let Some(field) = iter.next() {
                let value = iter.next();
                let f = match field {
                    RedisValue::String(s) | RedisValue::Status(s) => s.clone(),
                    other => other.to_display_string(),
                };
                let v = match value {
                    Some(RedisValue::String(s)) | Some(RedisValue::Status(s)) => s.clone(),
                    Some(other) => other.to_display_string(),
                    None => String::new(),
                };
                result.push((f, v));
            }
            result
        }
        _ => vec![],
    }
}

// --- HyperLogLog commands ---

#[tauri::command]
pub async fn get_hyperloglog_count(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<i64, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let val = client.execute("PFCOUNT", vec![key]).await?;
    parse_i64(val)
}

// --- Geo commands ---

#[tauri::command]
pub async fn get_geo_members(
    connection_id: String,
    key: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<GeoMember>, String> {
    let map = manager.lock().await;
    let client = map
        .get(&connection_id)
        .ok_or_else(|| format!("connection '{}' not found", connection_id))?;
    let members_val = client.execute("ZRANGE", vec![key.clone(), "0".into(), "-1".into()]).await?;
    let members = parse_string_array(members_val)?;
    if members.is_empty() {
        return Ok(vec![]);
    }
    let mut geopos_args = vec![key.clone()];
    for m in &members {
        geopos_args.push(m.clone());
    }
    let val = client.execute("GEOPOS", geopos_args).await?;
    let mut result = Vec::new();
    match val {
        RedisValue::Array(positions) => {
            for (i, pos) in positions.into_iter().enumerate() {
                let member = members[i].clone();
                match pos {
                    RedisValue::Array(coords) if coords.len() >= 2 => {
                        let lon = match &coords[0] {
                            RedisValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                            RedisValue::Float(f) => *f,
                            _ => 0.0,
                        };
                        let lat = match &coords[1] {
                            RedisValue::String(s) => s.parse::<f64>().unwrap_or(0.0),
                            RedisValue::Float(f) => *f,
                            _ => 0.0,
                        };
                        let score_val = client
                            .execute("ZSCORE", vec![key.clone(), member.clone()])
                            .await;
                        let score = match score_val {
                            Ok(RedisValue::String(s)) => s.parse::<f64>().unwrap_or(0.0),
                            Ok(RedisValue::Float(f)) => f,
                            _ => 0.0,
                        };
                        result.push(GeoMember { member, longitude: lon, latitude: lat, score });
                    }
                    RedisValue::Nil => {
                        result.push(GeoMember { member, longitude: 0.0, latitude: 0.0, score: 0.0 });
                    }
                    _ => {}
                }
            }
        }
        RedisValue::Error(e) => return Err(e),
        _ => {}
    }
    Ok(result)
}
