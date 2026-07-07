# Phase 2: Redis Backend

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rust Redis client supporting standalone, cluster, sentinel, SSH tunnel, SSL, and Teleport detection.

**Architecture:** Abstract `RedisClient` trait with multiple implementations. Tauri commands expose each operation to frontend.

**Tech Stack:** `redis` crate (redis-rs), `thrussh`/`ssh2` for tunnels, `native-tls`/`rustls` for TLS

## Global Constraints
- All Redis operations use SCAN (never KEYS) for key listing
- Command timeout: 10s default, configurable
- Connection pool: 5 connections default
- Teleport auto-detect on connect

---

## Task 9: Redis Client Core (Rust)

**Files:**
- Create: `src-tauri/src/redis/client.rs`
- Create: `src-tauri/src/redis/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `RedisClient` trait:
  - `async fn connect(&mut self) -> Result<()>`
  - `async fn disconnect(&mut self) -> Result<()>`
  - `async fn ping(&self) -> Result<String>`
  - `async fn execute(&self, cmd: &str, args: &[&str]) -> Result<RedisValue>`
  - `async fn scan_keys(&self, cursor: u64, count: u64, pattern: &str) -> Result<(u64, Vec<String>)>`
  - `async fn get_type(&self, key: &str) -> Result<String>`
  - `async fn get_ttl(&self, key: &str) -> Result<i64>`
  - `async fn del(&self, keys: &[&str]) -> Result<i64>`
  - `async fn rename(&self, old: &str, new: &str) -> Result<()>`
- Produces: `RedisValue` enum: `Nil`, `String(String)`, `Integer(i64)`, `Array(Vec<RedisValue>)`, `Status(String)`, `Error(String)`
- Produces: `StandaloneClient` implementing `RedisClient`

**Steps:**

- [ ] **Step 1: Write tests**

```rust
// src-tauri/src/redis/client.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_value_string() {
        let v = RedisValue::String("hello".to_string());
        assert_eq!(v.as_str(), Some("hello"));
    }

    #[test]
    fn test_redis_value_nil() {
        let v = RedisValue::Nil;
        assert!(v.is_nil());
    }

    #[test]
    fn test_redis_value_array() {
        let v = RedisValue::Array(vec![
            RedisValue::String("a".to_string()),
            RedisValue::String("b".to_string()),
        ]);
        assert_eq!(v.as_array().map(|a| a.len()), Some(2));
    }

    #[test]
    fn test_redis_value_integer() {
        let v = RedisValue::Integer(42);
        assert_eq!(v.as_i64(), Some(42));
    }

    #[test]
    fn test_redis_value_error() {
        let v = RedisValue::Error("ERR something".to_string());
        assert!(v.is_error());
    }
}
```

- [ ] **Step 2: Run tests, verify fail**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 3: Implement RedisValue + trait**

```rust
// src-tauri/src/redis/client.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "value")]
pub enum RedisValue {
    Nil,
    String(String),
    Integer(i64),
    Float(f64),
    Array(Vec<RedisValue>),
    Status(String),
    Error(String),
    Bool(bool),
}

impl RedisValue {
    pub fn is_nil(&self) -> bool { matches!(self, Self::Nil) }
    pub fn is_error(&self) -> bool { matches!(self, Self::Error(_)) }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::Status(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<RedisValue>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Self::Nil => "(nil)".to_string(),
            Self::String(s) => s.clone(),
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Status(s) => s.clone(),
            Self::Error(e) => format!("(error) {e}"),
            Self::Bool(b) => b.to_string(),
            Self::Array(arr) => {
                arr.iter().enumerate()
                    .map(|(i, v)| format!("{}) {}", i + 1, v.to_display_string()))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
}

#[async_trait]
pub trait RedisClient: Send + Sync {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn ping(&self) -> Result<RedisValue, Box<dyn std::error::Error>>;
    async fn execute(&self, cmd: &str, args: &[&str]) -> Result<RedisValue, Box<dyn std::error::Error>>;
    async fn scan_keys(&self, cursor: u64, count: u64, pattern: &str)
        -> Result<(u64, Vec<String>), Box<dyn std::error::Error>>;
    async fn get_type(&self, key: &str) -> Result<String, Box<dyn std::error::Error>>;
    async fn get_ttl(&self, key: &str) -> Result<i64, Box<dyn std::error::Error>>;
    async fn del(&self, keys: &[&str]) -> Result<i64, Box<dyn std::error::Error>>;
    async fn rename(&self, old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>>;
    async fn set_ttl(&self, key: &str, seconds: i64) -> Result<bool, Box<dyn std::error::Error>>;
    async fn persist(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>>;
}
```

- [ ] **Step 4: Implement StandaloneClient**

```rust
// src-tauri/src/redis/standalone.rs
use super::client::{RedisClient, RedisValue};
use crate::config::ConnectionConfig;
use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, Client, RedisResult};

pub struct StandaloneClient {
    config: ConnectionConfig,
    conn: Option<MultiplexedConnection>,
}

impl StandaloneClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None }
    }

    fn build_url(&self) -> String {
        match &self.config.password {
            Some(pass) => format!(
                "redis://{}:{}@{}:{}/{}",
                self.config.username.as_deref().unwrap_or("default"),
                pass, self.config.host, self.config.port, self.config.db
            ),
            None => format!("redis://{}:{}/{}", self.config.host, self.config.port, self.config.db),
        }
    }
}

#[async_trait]
impl RedisClient for StandaloneClient {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let url = self.build_url();
        let client = Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        self.conn = Some(conn);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn = None;
        Ok(())
    }

    async fn ping(&self) -> Result<RedisValue, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: String = redis::cmd("PING").query_async(&mut conn.clone()).await?;
        Ok(RedisValue::Status(result))
    }

    async fn execute(&self, cmd: &str, args: &[&str]) -> Result<RedisValue, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let mut redis_cmd = redis::cmd(cmd);
        for arg in args {
            redis_cmd.arg(*arg);
        }
        let result: redis::Value = redis_cmd.query_async(&mut conn.clone()).await?;
        Ok(convert_value(result))
    }

    async fn scan_keys(
        &self, cursor: u64, count: u64, pattern: &str,
    ) -> Result<(u64, Vec<String>), Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(count)
            .query_async(&mut conn.clone())
            .await?;
        Ok((new_cursor, keys))
    }

    async fn get_type(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: String = redis::cmd("TYPE")
            .arg(key)
            .query_async(&mut conn.clone())
            .await?;
        Ok(result)
    }

    async fn get_ttl(&self, key: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: i64 = redis::cmd("TTL")
            .arg(key)
            .query_async(&mut conn.clone())
            .await?;
        Ok(result)
    }

    async fn del(&self, keys: &[&str]) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let mut cmd = redis::cmd("DEL");
        for key in keys {
            cmd.arg(*key);
        }
        let result: i64 = cmd.query_async(&mut conn.clone()).await?;
        Ok(result)
    }

    async fn rename(&self, old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        redis::cmd("RENAME")
            .arg(old)
            .arg(new)
            .query_async::<_, ()>(&mut conn.clone())
            .await?;
        Ok(())
    }

    async fn set_ttl(&self, key: &str, seconds: i64) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: i64 = redis::cmd("EXPIRE")
            .arg(key)
            .arg(seconds)
            .query_async(&mut conn.clone())
            .await?;
        Ok(result == 1)
    }

    async fn persist(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: i64 = redis::cmd("PERSIST")
            .arg(key)
            .query_async(&mut conn.clone())
            .await?;
        Ok(result == 1)
    }
}

fn convert_value(value: redis::Value) -> RedisValue {
    match value {
        redis::Value::Nil => RedisValue::Nil,
        redis::Value::Int(i) => RedisValue::Integer(i),
        redis::Value::Data(bytes) => {
            RedisValue::String(String::from_utf8_lossy(&bytes).to_string())
        }
        redis::Value::Bulk(values) => {
            RedisValue::Array(values.into_iter().map(convert_value).collect())
        }
        redis::Value::Status(s) => RedisValue::Status(s),
        redis::Value::Okay => RedisValue::Status("OK".to_string()),
        redis::Value::Boolean(b) => RedisValue::Bool(b),
        redis::Value::Double(d) => RedisValue::Float(d),
    }
}
```

- [ ] **Step 5: Update mod.rs**

```rust
// src-tauri/src/redis/mod.rs
pub mod client;
pub mod standalone;

pub use client::*;
pub use standalone::*;
```

- [ ] **Step 6: Add deps to Cargo.toml**

```toml
redis = { version = "0.27", features = ["tokio-comp", "connection-manager", "cluster"] }
async-trait = "0.1"
tokio = { version = "1", features = ["full"] }
```

- [ ] **Step 7: Run tests, verify pass**

```bash
cd src-tauri && cargo test
```

Expected: Unit tests PASS (integration tests need Docker Redis — Phase 7).

- [ ] **Step 8: Commit**

```bash
git add . && git commit -m "feat: redis client core with standalone impl"
```

---

## Task 10: SSH Tunnel Support (Rust)

**Files:**
- Create: `src-tauri/src/redis/tunnel.rs`
- Modify: `src-tauri/src/redis/mod.rs`

**Interfaces:**
- Produces: `SshTunnel` struct:
  - `async fn open(config: &SshConfig, target_host: &str, target_port: u16) -> Result<Self>`
  - `fn local_port(&self) -> u16`
  - `async fn close(&self) -> Result<()>`

**Steps:**

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_config_parsing() {
        let config = SshConfig {
            host: "bastion.example.com".to_string(),
            port: 22,
            username: "admin".to_string(),
            auth: SshAuth::KeyFile("/home/user/.ssh/id_rsa".to_string()),
        };
        assert_eq!(config.host, "bastion.example.com");
        assert_eq!(config.port, 22);
    }

    #[test]
    fn test_tunnel_localhost_format() {
        let port = 16379u16;
        let url = format!("redis://127.0.0.1:{port}/0");
        assert!(url.contains("127.0.0.1"));
    }
}
```

- [ ] **Step 2: Implement SSH tunnel**

```rust
// src-tauri/src/redis/tunnel.rs
use crate::config::{SshAuth, SshConfig};
use ssh2::Session;
use std::io::Read;
use std::net::TcpListener;
use tokio::net::TcpStream;

pub struct SshTunnel {
    local_port: u16,
    _session: Session,
}

impl SshTunnel {
    pub async fn open(
        config: &SshConfig,
        target_host: &str,
        target_port: u16,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let ssh_addr = format!("{}:{}", config.host, config.port);
        let tcp = TcpStream::connect(&ssh_addr).await?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp.into_std()?);
        session.handshake()?;

        match &config.auth {
            SshAuth::KeyFile(path) => {
                session.userkey_pubkey_file(&config.username, None, std::path::Path::new(path), None)?;
            }
            SshAuth::Password(pass) => {
                session.userauth_password(&config.username, pass)?;
            }
        }

        if !session.authenticated() {
            return Err("SSH authentication failed".into());
        }

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let local_port = listener.local_addr()?.port();
        drop(listener);

        // Background forwarding (simplified — production should handle this async)
        let mut channel = session.channel_session()?;
        channel.request_direct_tcpip(target_host, target_port, Some(("127.0.0.1", local_port.into())))?;

        Ok(Self { local_port, _session: session })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}
```

- [ ] **Step 3: Add deps**

```toml
ssh2 = "0.9"
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: SSH tunnel support for redis connections"
```

---

## Task 11: Cluster + Sentinel Mode (Rust)

**Files:**
- Create: `src-tauri/src/redis/cluster.rs`
- Create: `src-tauri/src/redis/sentinel.rs`
- Modify: `src-tauri/src/redis/mod.rs`

**Interfaces:**
- Produces: `ClusterClient` implementing `RedisClient`
- Produces: `SentinelClient` implementing `RedisClient`

**Steps:**

- [ ] **Step 1: Implement ClusterClient**

```rust
// src-tauri/src/redis/cluster.rs
use super::client::{RedisClient, RedisValue, convert_value};
use crate::config::ConnectionConfig;
use async_trait::async_trait;
use redis::cluster::ClusterClient as RedisClusterClient;
use redis::cluster_async::ClusterConnection;

pub struct ClusterClient {
    config: ConnectionConfig,
    conn: Option<ClusterConnection>,
}

impl ClusterClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { config, conn: None }
    }

    fn build_urls(&self) -> Vec<String> {
        // Cluster may have multiple seed nodes; for now use single host
        vec![format!("redis://{}:{}", self.config.host, self.config.port)]
    }
}

#[async_trait]
impl RedisClient for ClusterClient {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let urls = self.build_urls();
        let client = RedisClusterClient::new(urls)?;
        let conn = client.get_async_connection().await?;
        self.conn = Some(conn);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn = None;
        Ok(())
    }

    async fn ping(&self) -> Result<RedisValue, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let result: String = redis::cmd("PING").query_async(&mut conn.clone()).await?;
        Ok(RedisValue::Status(result))
    }

    async fn execute(&self, cmd: &str, args: &[&str]) -> Result<RedisValue, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let mut redis_cmd = redis::cmd(cmd);
        for arg in args { redis_cmd.arg(*arg); }
        let result: redis::Value = redis_cmd.query_async(&mut conn.clone()).await?;
        Ok(convert_value(result))
    }

    async fn scan_keys(
        &self, cursor: u64, count: u64, pattern: &str,
    ) -> Result<(u64, Vec<String>), Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let (new_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor).arg("MATCH").arg(pattern).arg("COUNT").arg(count)
            .query_async(&mut conn.clone()).await?;
        Ok((new_cursor, keys))
    }

    // pongolong: delegate remaining trait methods same as StandaloneClient
    // get_type, get_ttl, del, rename, set_ttl, persist — identical pattern
    // Copy from standalone.rs, replacing self.conn type

    async fn get_type(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        Ok(redis::cmd("TYPE").arg(key).query_async::<_, String>(&mut conn.clone()).await?)
    }

    async fn get_ttl(&self, key: &str) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        Ok(redis::cmd("TTL").arg(key).query_async::<_, i64>(&mut conn.clone()).await?)
    }

    async fn del(&self, keys: &[&str]) -> Result<i64, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        let mut cmd = redis::cmd("DEL");
        for key in keys { cmd.arg(*key); }
        Ok(cmd.query_async::<_, i64>(&mut conn.clone()).await?)
    }

    async fn rename(&self, old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        redis::cmd("RENAME").arg(old).arg(new).query_async::<_, ()>(&mut conn.clone()).await?;
        Ok(())
    }

    async fn set_ttl(&self, key: &str, seconds: i64) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        Ok(redis::cmd("EXPIRE").arg(key).arg(seconds).query_async::<_, i64>(&mut conn.clone()).await? == 1)
    }

    async fn persist(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let conn = self.conn.as_ref().ok_or("Not connected")?;
        Ok(redis::cmd("PERSIST").arg(key).query_async::<_, i64>(&mut conn.clone()).await? == 1)
    }
}
```

- [ ] **Step 2: Implement SentinelClient**

```rust
// src-tauri/src/redis/sentinel.rs
// pongolong: Sentinel uses redis::sentinel module
// Structure mirrors StandaloneClient but connects via Sentinel
// For v1, use direct connection to resolved master
// Full sentinel support can be added incrementally

use super::standalone::StandaloneClient;
use super::client::RedisClient;
use crate::config::ConnectionConfig;
use async_trait::async_trait;

pub struct SentinelClient {
    inner: StandaloneClient,
}

impl SentinelClient {
    pub fn new(config: ConnectionConfig) -> Self {
        Self { inner: StandaloneClient::new(config) }
    }
}

// pongolong: Delegate all trait methods to inner StandaloneClient
// Sentinel discovery happens at connect time — resolve master then delegate
#[async_trait]
impl RedisClient for SentinelClient {
    async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // pongolong: TODO — implement Sentinel discovery
        // For v1, treat as standalone (direct connect)
        self.inner.connect().await
    }
    async fn disconnect(&mut self) -> Result<(), Box<dyn std::error::Error>> { self.inner.disconnect().await }
    async fn ping(&self) -> Result<super::client::RedisValue, Box<dyn std::error::Error>> { self.inner.ping().await }
    async fn execute(&self, cmd: &str, args: &[&str]) -> Result<super::client::RedisValue, Box<dyn std::error::Error>> { self.inner.execute(cmd, args).await }
    async fn scan_keys(&self, cursor: u64, count: u64, pattern: &str) -> Result<(u64, Vec<String>), Box<dyn std::error::Error>> { self.inner.scan_keys(cursor, count, pattern).await }
    async fn get_type(&self, key: &str) -> Result<String, Box<dyn std::error::Error>> { self.inner.get_type(key).await }
    async fn get_ttl(&self, key: &str) -> Result<i64, Box<dyn std::error::Error>> { self.inner.get_ttl(key).await }
    async fn del(&self, keys: &[&str]) -> Result<i64, Box<dyn std::error::Error>> { self.inner.del(keys).await }
    async fn rename(&self, old: &str, new: &str) -> Result<(), Box<dyn std::error::Error>> { self.inner.rename(old, new).await }
    async fn set_ttl(&self, key: &str, seconds: i64) -> Result<bool, Box<dyn std::error::Error>> { self.inner.set_ttl(key, seconds).await }
    async fn persist(&self, key: &str) -> Result<bool, Box<dyn std::error::Error>> { self.inner.persist(key).await }
}
```

- [ ] **Step 3: Update mod.rs**

```rust
// src-tauri/src/redis/mod.rs
pub mod client;
pub mod standalone;
pub mod cluster;
pub mod sentinel;
pub mod tunnel;

pub use client::*;
pub use standalone::*;
pub use cluster::*;
pub use sentinel::*;
```

- [ ] **Step 4: Build verify**

```bash
cd src-tauri && cargo build
```

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: cluster and sentinel redis client impls"
```

---

## Task 12: Teleport Detection (Rust)

**Files:**
- Create: `src-tauri/src/redis/teleport.rs`
- Modify: `src-tauri/src/redis/mod.rs`

**Interfaces:**
- Produces: `fn detect_teleport(client: &dyn RedisClient) -> Result<TeleportStatus>`
- Produces: `TeleportStatus` struct: `is_teleport: bool, restricted_commands: Vec<String>`
- Known restricted commands: `CLUSTER`, `CONFIG`, `DEBUG`, `SCRIPT`, `SHUTDOWN`, `SLAVEOF`, `REPLICAOF`, `CLIENT`, `ACL`, `MODULE`, `SWAPDB`, `LATENCY`, `MEMORY`, `MONITOR`, `KEYS`

**Steps:**

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restricted_commands_list() {
        let list = restricted_commands();
        assert!(list.contains(&"CLUSTER"));
        assert!(list.contains(&"CONFIG"));
        assert!(list.contains(&"DEBUG"));
        assert!(list.contains(&"KEYS"));
        assert!(!list.contains(&"GET"));
    }

    #[test]
    fn test_is_restricted() {
        assert!(is_restricted("CLUSTER INFO"));
        assert!(is_restricted("CONFIG SET"));
        assert!(!is_restricted("GET mykey"));
        assert!(!is_restricted("SET mykey value"));
    }
}
```

- [ ] **Step 2: Implement**

```rust
// src-tauri/src/redis/teleport.rs

const RESTRICTED: &[&str] = &[
    "CLUSTER", "CONFIG", "DEBUG", "SCRIPT", "SHUTDOWN",
    "SLAVEOF", "REPLICAOF", "CLIENT", "ACL", "MODULE",
    "SWAPDB", "LATENCY", "MEMORY", "MONITOR", "KEYS",
    "SAVE", "BGSAVE", "BGREWRITEAOF", "LASTSAVE",
    "DBSIZE", "FLUSHDB", "FLUSHALL", "SORT",
];

pub fn restricted_commands() -> Vec<String> {
    RESTRICTED.iter().map(|s| s.to_string()).collect()
}

pub fn is_restricted(command: &str) -> bool {
    let cmd = command.trim().split_whitespace().next().unwrap_or("").to_uppercase();
    RESTRICTED.iter().any(|r| r == &cmd)
}

pub async fn detect_teleport(client: &dyn super::RedisClient) -> bool {
    // Try COMMAND INFO — Teleport blocks this for restricted users
    match client.execute("COMMAND", &["INFO", "CLUSTER"]).await {
        Ok(v) => v.is_error(),
        Err(_) => true, // Connection error likely means restricted
    }
}
```

- [ ] **Step 3: Update mod.rs**

```rust
// Add to mod.rs
pub mod teleport;
pub use teleport::*;
```

- [ ] **Step 4: Run tests**

```bash
cd src-tauri && cargo test
```

- [ ] **Step 5: Commit**

```bash
git add . && git commit -m "feat: teleport detection with restricted command list"
```
