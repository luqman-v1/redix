use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, cmd, IntoConnectionInfo, Value};
use redis::sentinel::Sentinel;

use crate::config::connection::ConnectionConfig;

use super::client::RedisClient;
use super::types::RedisValue;
use super::standalone::{convert_value, redis_value_to_string};

/// Redis Sentinel client that performs master discovery.
pub struct SentinelClient {
    config: ConnectionConfig,
    conn: Option<MultiplexedConnection>,
}

impl SentinelClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None }
    }

    fn build_url(&self) -> String {
        let scheme = if self.config.use_ssl || self.config.ssl.is_some() { "rediss://" } else { "redis://" };
        let mut url = String::from(scheme);

        match (&self.config.username, &self.config.password) {
            (Some(user), Some(pass)) => {
                url.push_str(&format!("{}:{}@", user, pass));
            }
            (None, Some(pass)) => {
                url.push_str(&format!(":{}@", pass));
            }
            _ => {}
        }

        url.push_str(&format!("{}:{}", self.config.host, self.config.port));
        if let Some(ssl) = &self.config.ssl {
            if ssl.skip_verify {
                url.push_str("#insecure");
            }
        }
        url
    }
}

#[async_trait]
impl RedisClient for SentinelClient {
    async fn connect(&mut self) -> Result<(), String> {
        let url = self.build_url();
        let info = url.into_connection_info().map_err(|e| format!("invalid url: {}", e))?;
        
        let mut sentinel = Sentinel::build(vec![info])
            .map_err(|e| format!("sentinel client creation failed: {}", e))?;
            
        // Use "mymaster" as the default master name for Sentinel
        let master_name = self.config.sentinel_master_name.as_deref().unwrap_or("mymaster");
        
        let client = sentinel.async_master_for(master_name, None)
            .await
            .map_err(|e| format!("sentinel master discovery failed: {}", e))?;

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

    async fn execute(&self, command: &str, args: Vec<String>) -> Result<RedisValue, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();

        // Check for read-only mode safety
        if self.config.readonly {
            let upper = command.to_uppercase();
            if crate::redis::UNSAFE_CMDS.contains(&upper.as_str()) {
                return Err(format!("Command {} not allowed in read-only mode", upper));
            }
        }

        let mut cmd = cmd(command);
        for arg in args {
            cmd.arg(arg);
        }

        let result: Value = cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("execute failed: {}", e))?;

        Ok(convert_value(result))
    }

    async fn execute_pipeline(&self, cmds: Vec<(String, Vec<String>)>) -> Result<Vec<RedisValue>, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut pipe = redis::pipe();
        for (command, args) in cmds {
            let mut redis_cmd = cmd(&command);
            for arg in &args {
                redis_cmd.arg(arg);
            }
            pipe.add_command(redis_cmd);
        }
        let values: Vec<Value> = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("pipeline failed: {}", e))?;
        Ok(values.into_iter().map(convert_value).collect())
    }

    async fn scan_keys(
        &self,
        cursor: u64,
        count: u64,
        pattern: Option<&str>,
    ) -> Result<(u64, Vec<String>), String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();

        let mut cmd = cmd("SCAN");
        cmd.arg(cursor).arg("COUNT").arg(count);
        if let Some(p) = pattern {
            cmd.arg("MATCH").arg(p);
        }

        let result: Value = cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("scan failed: {}", e))?;

        match result {
            Value::Array(items) if items.len() == 2 => {
                let next_cursor = match &items[0] {
                    Value::BulkString(bytes) => String::from_utf8(bytes.clone())
                        .unwrap_or_else(|_| "0".to_string())
                        .parse::<u64>()
                        .unwrap_or(0),
                    _ => 0,
                };

                let mut keys = Vec::new();
                if let Value::Array(key_items) = &items[1] {
                    for item in key_items {
                        if let Value::BulkString(bytes) = item {
                            if let Ok(key) = String::from_utf8(bytes.clone()) {
                                keys.push(key);
                            }
                        }
                    }
                }

                Ok((next_cursor, keys))
            }
            _ => Err("Invalid response format from SCAN".to_string()),
        }
    }

    async fn get_type(&self, key: &str) -> Result<String, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: Value = cmd("TYPE")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("TYPE failed: {}", e))?;
        redis_value_to_string(result)
    }

    async fn get_ttl(&self, key: &str) -> Result<i64, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: Value = cmd("TTL")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("TTL failed: {}", e))?;

        match result {
            Value::Int(ttl) => Ok(ttl),
            _ => Err("Invalid response from TTL".to_string()),
        }
    }

    async fn del(&self, keys: Vec<&str>) -> Result<i64, String> {
        if self.config.readonly {
            return Err("DEL not allowed in read-only mode".to_string());
        }

        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut command = cmd("DEL");
        for key in keys {
            command.arg(key);
        }

        let result: Value = command
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("DEL failed: {}", e))?;

        match result {
            Value::Int(count) => Ok(count),
            _ => Err("Invalid response from DEL".to_string()),
        }
    }

    async fn rename(&self, old: &str, new: &str) -> Result<(), String> {
        if self.config.readonly {
            return Err("RENAME not allowed in read-only mode".to_string());
        }
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let _result: Value = cmd("RENAME")
            .arg(old)
            .arg(new)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("RENAME failed: {}", e))?;
        Ok(())
    }

    async fn set_ttl(&self, key: &str, seconds: u64) -> Result<bool, String> {
        if self.config.readonly {
            return Err("EXPIRE not allowed in read-only mode".to_string());
        }
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: Value = cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("EXPIRE failed: {}", e))?;
        match result {
            Value::Int(1) => Ok(true),
            Value::Int(0) => Ok(false),
            _ => Err("Invalid response from EXPIRE".to_string()),
        }
    }

    async fn persist(&self, key: &str) -> Result<bool, String> {
        if self.config.readonly {
            return Err("PERSIST not allowed in read-only mode".to_string());
        }
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let result: Value = cmd("PERSIST")
            .arg(key)
            .query_async(&mut conn)
            .await
            .map_err(|e| format!("PERSIST failed: {}", e))?;
        match result {
            Value::Int(1) => Ok(true),
            Value::Int(0) => Ok(false),
            _ => Err("Invalid response from PERSIST".to_string()),
        }
    }
}
