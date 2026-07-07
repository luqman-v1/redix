use async_trait::async_trait;

use crate::config::ConnectionConfig;

use super::client::{RedisClient, RedisValue};
use super::standalone::StandaloneClient;

/// Redis Sentinel client. v1: treats sentinel config as direct connect.
/// Full sentinel discovery can be added later.
pub struct SentinelClient {
    inner: StandaloneClient,
}

impl SentinelClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            inner: StandaloneClient::new(config),
        }
    }
}

#[async_trait]
impl RedisClient for SentinelClient {
    async fn connect(&mut self) -> Result<(), String> {
        self.inner.connect().await
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.inner.disconnect().await
    }

    async fn ping(&self) -> Result<bool, String> {
        self.inner.ping().await
    }

    async fn execute(&self, cmd: &str, args: Vec<String>) -> Result<RedisValue, String> {
        self.inner.execute(cmd, args).await
    }

    async fn scan_keys(
        &self,
        cursor: u64,
        count: u64,
        pattern: Option<&str>,
    ) -> Result<(u64, Vec<String>), String> {
        self.inner.scan_keys(cursor, count, pattern).await
    }

    async fn get_type(&self, key: &str) -> Result<String, String> {
        self.inner.get_type(key).await
    }

    async fn get_ttl(&self, key: &str) -> Result<i64, String> {
        self.inner.get_ttl(key).await
    }

    async fn del(&self, keys: Vec<&str>) -> Result<i64, String> {
        self.inner.del(keys).await
    }

    async fn rename(&self, old: &str, new: &str) -> Result<(), String> {
        self.inner.rename(old, new).await
    }

    async fn set_ttl(&self, key: &str, seconds: u64) -> Result<bool, String> {
        self.inner.set_ttl(key, seconds).await
    }

    async fn persist(&self, key: &str) -> Result<bool, String> {
        self.inner.persist(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionType;

    #[test]
    fn test_sentinel_client_new() {
        let mut config = ConnectionConfig::new("sentinel-test", "127.0.0.1", 26379);
        config.connection_type = ConnectionType::Sentinel;
        let client = SentinelClient::new(config);
        // inner StandaloneClient starts disconnected
        // just verify construction succeeds
        drop(client);
    }
}
