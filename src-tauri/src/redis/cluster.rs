use async_trait::async_trait;
use redis::cluster_async::ClusterConnection;
use redis::{cluster::ClusterClient as RedisClusterClient, FromRedisValue, Value};

use crate::config::ConnectionConfig;

use super::client::{RedisClient, RedisValue};
use super::standalone::{convert_value, redis_value_to_string};

/// Redis Cluster client backed by a cluster connection.
pub struct ClusterClient {
    config: ConnectionConfig,
    conn: Option<ClusterConnection>,
}

impl ClusterClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None }
    }

    fn build_urls(&self) -> Vec<String> {
        let mut url = String::from("redis://");

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
        vec![url]
    }
}

#[async_trait]
impl RedisClient for ClusterClient {
    async fn connect(&mut self) -> Result<(), String> {
        let urls = self.build_urls();
        let client = RedisClusterClient::new(urls)
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
