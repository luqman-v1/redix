use async_trait::async_trait;
use redis::cluster_async::ClusterConnection;
use redis::{FromRedisValue, Value};

use crate::config::ConnectionConfig;
use crate::redis::tls::build_tls_certificates;

use super::client::RedisClient;
use super::types::RedisValue;
use super::standalone::{convert_value, redis_value_to_string};

/// Redis Cluster client backed by a cluster connection.
pub struct ClusterClient {
    config: ConnectionConfig,
    conn: Option<ClusterConnection>,
    /// Cached standalone connection to entrypoint node for keyless commands (INFO, DBSIZE)
    fallback_conn: tokio::sync::Mutex<Option<redis::aio::MultiplexedConnection>>,
}

impl ClusterClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None, fallback_conn: tokio::sync::Mutex::new(None) }
    }

    fn build_urls(&self) -> Vec<String> {
        let use_ssl = self.config.use_ssl || self.config.ssl.is_some();
        let scheme = if use_ssl { "rediss://" } else { "redis://" };
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
        vec![url]
    }
}

#[async_trait]
impl RedisClient for ClusterClient {
    async fn connect(&mut self) -> Result<(), String> {
        let urls = self.build_urls();
        let mut builder = redis::cluster::ClusterClientBuilder::new(urls).retries(5);
        if self.config.readonly {
            builder = builder.read_from_replicas();
        }
        
        let use_ssl = self.config.use_ssl || self.config.ssl.is_some();
        if use_ssl {
            let certs = build_tls_certificates(&self.config)?;
            builder = builder.certs(certs);
        }

        let client = builder
            .build()
            .map_err(|e| format!("cluster client creation failed: {}", e))?;
        let conn = client
            .get_async_connection()
            .await
            .map_err(|e| format!("cluster connection failed: {}", e))?;
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
        if self.config.readonly {
            let upper_cmd = cmd.to_uppercase();
            if crate::redis::UNSAFE_CMDS.contains(&upper_cmd.as_str()) {
                return Err("Connection is in Read-Only mode".into());
            }
        }
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut redis_cmd = redis::cmd(cmd);
        let upper_cmd = cmd.to_uppercase();
        for arg in &args {
            redis_cmd.arg(arg);
        }
        
        // Intercept cluster-wide or keyless commands that cluster_async struggles with
        if upper_cmd == "INFO" || upper_cmd == "DBSIZE" {
            let mut fb_guard = self.fallback_conn.lock().await;
            if fb_guard.is_none() {
                let urls = self.build_urls();
                let fallback_client = redis::Client::open(urls[0].clone())
                    .map_err(|err| format!("fallback client failed: {}", err))?;
                let fb_conn = fallback_client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|err| format!("fallback conn failed: {}", err))?;
                *fb_guard = Some(fb_conn);
            }
            let mut fb = fb_guard.as_ref().unwrap().clone();
            drop(fb_guard);
            let value: Value = redis_cmd
                .query_async(&mut fb)
                .await
                .map_err(|err| format!("command failed on fallback: {}", err))?;
            return Ok(convert_value(value));
        }

        let value: Value = match redis_cmd.query_async(&mut conn).await {
            Ok(v) => v,
            Err(e) => return Err(format!("command failed: {}", e)),
        };
        Ok(convert_value(value))
    }

    async fn execute_pipeline(&self, cmds: Vec<(String, Vec<String>)>) -> Result<Vec<RedisValue>, String> {
        let conn = self.conn.as_ref().ok_or("not connected")?;
        let mut conn = conn.clone();
        let mut pipe = redis::pipe();
        for (cmd, args) in cmds {
            let mut redis_cmd = redis::cmd(&cmd);
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
        if self.config.readonly {
            return Err("Connection is in Read-Only mode".into());
        }
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
        if self.config.readonly {
            return Err("Connection is in Read-Only mode".into());
        }
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
        if self.config.readonly {
            return Err("Connection is in Read-Only mode".into());
        }
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
        if self.config.readonly {
            return Err("Connection is in Read-Only mode".into());
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionType;

    #[test]
    fn test_cluster_client_new() {
        let mut config = ConnectionConfig::new("cluster-test", "127.0.0.1", 7000);
        config.connection_type = ConnectionType::Cluster;
        let client = ClusterClient::new(config.clone());
        assert!(client.conn.is_none());
        assert_eq!(client.config.name, "cluster-test");
        assert_eq!(client.config.host, "127.0.0.1");
        assert_eq!(client.config.port, 7000);
    }

    #[test]
    fn test_cluster_client_build_urls() {
        let mut config = ConnectionConfig::new("cluster-test", "10.0.0.1", 7001);
        config.connection_type = ConnectionType::Cluster;
        let client = ClusterClient::new(config);
        let urls = client.build_urls();
        assert_eq!(urls, vec!["redis://10.0.0.1:7001"]);
    }

    #[test]
    fn test_cluster_client_build_urls_with_auth() {
        let mut config = ConnectionConfig::new("cluster-auth", "10.0.0.1", 7001);
        config.connection_type = ConnectionType::Cluster;
        config.username = Some("admin".to_string());
        config.password = Some("secret".to_string());
        let client = ClusterClient::new(config);
        let urls = client.build_urls();
        assert_eq!(urls, vec!["redis://admin:secret@10.0.0.1:7001"]);
    }
}
