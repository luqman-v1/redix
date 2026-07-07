use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, Client, FromRedisValue, Value};

use crate::config::ConnectionConfig;

use super::client::{RedisClient, RedisValue};

/// Standalone Redis client backed by a multiplexed connection.
pub struct StandaloneClient {
    config: ConnectionConfig,
    conn: Option<MultiplexedConnection>,
}

impl StandaloneClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None }
    }
}

fn build_url(config: &ConnectionConfig) -> String {
    let mut url = String::from("redis://");

    match (&config.username, &config.password) {
        (Some(user), Some(pass)) => {
            url.push_str(&format!("{}:{}@", user, pass));
        }
        (None, Some(pass)) => {
            url.push_str(&format!(":{}@", pass));
        }
        _ => {}
    }

    url.push_str(&format!("{}:{}/{}", config.host, config.port, config.db));
    url
}

fn convert_value(value: Value) -> RedisValue {
    match value {
        Value::Nil => RedisValue::Nil,
        Value::Int(n) => RedisValue::Integer(n),
        Value::Array(items) => {
            let converted: Vec<RedisValue> = items.into_iter().map(convert_value).collect();
            RedisValue::Array(converted)
        }
        Value::BulkString(bytes) => match String::from_utf8(bytes) {
            Ok(s) => RedisValue::String(s),
            Err(e) => RedisValue::Error(format!("invalid utf-8: {}", e)),
        },
        Value::SimpleString(s) => RedisValue::Status(s),
        Value::Okay => RedisValue::Status("OK".to_string()),
        Value::Double(f) => RedisValue::Float(f),
        Value::Boolean(b) => RedisValue::Bool(b),
        other => RedisValue::String(format!("{:?}", other)),
    }
}

fn redis_value_to_string(val: Value) -> Result<String, String> {
    match val {
        Value::BulkString(bytes) => String::from_utf8(bytes)
            .map_err(|e| format!("invalid utf-8: {}", e)),
        Value::SimpleString(s) => Ok(s),
        Value::Okay => Ok("OK".to_string()),
        Value::Int(n) => Ok(n.to_string()),
        _ => Err(format!("unexpected redis response type: {:?}", val)),
    }
}

#[async_trait]
impl RedisClient for StandaloneClient {
    async fn connect(&mut self) -> Result<(), String> {
        let url = build_url(&self.config);
        let client = Client::open(url).map_err(|e| format!("client creation failed: {}", e))?;
        let conn = client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| format!("connection failed: {}", e))?;
        self.conn = Some(conn);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.conn = None;
        Ok(())
    }

    async fn ping(&self) -> Result<bool, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("ping failed: {}", e))?;
        Ok(result == "PONG")
    }

    async fn execute(&self, cmd: &str, args: Vec<String>) -> Result<RedisValue, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut redis_cmd = redis::cmd(cmd);
        for arg in &args {
            redis_cmd.arg(arg);
        }
        let value: Value = redis_cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("command failed: {}", e))?;
        Ok(convert_value(value))
    }

    async fn scan_keys(
        &self,
        cursor: u64,
        count: u64,
        pattern: Option<&str>,
    ) -> Result<(u64, Vec<String>), String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();

        let mut cmd = redis::cmd("SCAN");
        cmd.arg(cursor).arg("COUNT").arg(count);
        if let Some(p) = pattern {
            cmd.arg("MATCH").arg(p);
        }

        let value: Value = cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("scan failed: {}", e))?;

        match value {
            Value::Array(items) if items.len() == 2 => {
                let new_cursor: u64 = FromRedisValue::from_redis_value(&items[0])
                    .map_err(|e| format!("cursor parse error: {}", e))?;
                let keys = match &items[1] {
                    Value::Array(key_items) => {
                        let mut result = Vec::with_capacity(key_items.len());
                        for k in key_items {
                            result.push(redis_value_to_string(k.clone())?);
                        }
                        result
                    }
                    _ => Vec::new(),
                };
                Ok((new_cursor, keys))
            }
            _ => Err("unexpected SCAN response".to_string()),
        }
    }

    async fn get_type(&self, key: &str) -> Result<String, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: Value = redis::cmd("TYPE")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("TYPE failed: {}", e))?;
        redis_value_to_string(result)
    }

    async fn get_ttl(&self, key: &str) -> Result<i64, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: i64 = redis::cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("TTL failed: {}", e))?;
        Ok(result)
    }

    async fn del(&self, keys: Vec<&str>) -> Result<i64, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut cmd = redis::cmd("DEL");
        for k in &keys {
            cmd.arg(k);
        }
        let result: i64 = cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("DEL failed: {}", e))?;
        Ok(result)
    }

    async fn rename(&self, old: &str, new: &str) -> Result<(), String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let _: Value = redis::cmd("RENAME")
            .arg(old)
            .arg(new)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("RENAME failed: {}", e))?;
        Ok(())
    }

    async fn set_ttl(&self, key: &str, seconds: u64) -> Result<bool, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: i64 = redis::cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("EXPIRE failed: {}", e))?;
        Ok(result == 1)
    }

    async fn persist(&self, key: &str) -> Result<bool, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: i64 = redis::cmd("PERSIST")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("PERSIST failed: {}", e))?;
        Ok(result == 1)
    }
}
