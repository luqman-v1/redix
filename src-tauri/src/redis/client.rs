use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Unified Redis value representation across all response types.
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
    pub fn is_nil(&self) -> bool {
        matches!(self, RedisValue::Nil)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, RedisValue::Error(_))
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            RedisValue::String(s) => Some(s),
            RedisValue::Status(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            RedisValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<RedisValue>> {
        match self {
            RedisValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            RedisValue::Nil => "(nil)".to_string(),
            RedisValue::String(s) => format!("\"{}\"", s),
            RedisValue::Integer(n) => n.to_string(),
            RedisValue::Float(f) => format!("{}", f),
            RedisValue::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", items.join(", "))
            }
            RedisValue::Status(s) => s.clone(),
            RedisValue::Error(e) => format!("(error) {}", e),
            RedisValue::Bool(b) => b.to_string(),
        }
    }
}

impl Default for RedisValue {
    fn default() -> Self {
        RedisValue::Nil
    }
}

/// Trait abstracting Redis client operations.
#[async_trait]
pub trait RedisClient: Send + Sync {
    /// Connect to the Redis server.
    async fn connect(&mut self) -> Result<(), String>;

    /// Disconnect from the Redis server.
    async fn disconnect(&mut self) -> Result<(), String>;

    /// Ping the server to check connectivity.
    async fn ping(&self) -> Result<bool, String>;

    /// Execute an arbitrary Redis command by name and arguments.
    async fn execute(&self, cmd: &str, args: Vec<String>) -> Result<RedisValue, String>;

    /// Scan keys using cursor-based pagination.
    /// Returns (next_cursor, keys).
    async fn scan_keys(
        &self,
        cursor: u64,
        count: u64,
        pattern: Option<&str>,
    ) -> Result<(u64, Vec<String>), String>;

    /// Get the type of a key.
    async fn get_type(&self, key: &str) -> Result<String, String>;

    /// Get TTL of a key in seconds. Returns -1 if no expiry, -2 if key does not exist.
    async fn get_ttl(&self, key: &str) -> Result<i64, String>;

    /// Delete one or more keys. Returns the number of keys removed.
    async fn del(&self, keys: Vec<&str>) -> Result<i64, String>;

    /// Rename a key.
    async fn rename(&self, old: &str, new: &str) -> Result<(), String>;

    /// Set TTL on a key in seconds.
    async fn set_ttl(&self, key: &str, seconds: u64) -> Result<bool, String>;

    /// Remove TTL from a key (PERSIST).
    async fn persist(&self, key: &str) -> Result<bool, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_value_string() {
        let val = RedisValue::String("hello".to_string());
        assert!(!val.is_nil());
        assert!(!val.is_error());
        assert_eq!(val.as_str(), Some("hello"));
        assert_eq!(val.as_i64(), None);
        assert_eq!(val.as_array(), None);
        assert_eq!(val.to_display_string(), "\"hello\"");
    }

    #[test]
    fn test_redis_value_nil() {
        let val = RedisValue::Nil;
        assert!(val.is_nil());
        assert!(!val.is_error());
        assert_eq!(val.as_str(), None);
        assert_eq!(val.as_i64(), None);
        assert_eq!(val.as_array(), None);
        assert_eq!(val.to_display_string(), "(nil)");
    }

    #[test]
    fn test_redis_value_array() {
        let val = RedisValue::Array(vec![
            RedisValue::String("a".to_string()),
            RedisValue::Integer(42),
        ]);
        assert!(!val.is_nil());
        assert!(!val.is_error());
        assert_eq!(val.as_str(), None);
        assert_eq!(val.as_i64(), None);
        assert!(val.as_array().is_some());
        assert_eq!(val.as_array().unwrap().len(), 2);
        assert_eq!(val.to_display_string(), "[\"a\", 42]");
    }

    #[test]
    fn test_redis_value_integer() {
        let val = RedisValue::Integer(123);
        assert!(!val.is_nil());
        assert!(!val.is_error());
        assert_eq!(val.as_str(), None);
        assert_eq!(val.as_i64(), Some(123));
        assert_eq!(val.as_array(), None);
        assert_eq!(val.to_display_string(), "123");
    }

    #[test]
    fn test_redis_value_error() {
        let val = RedisValue::Error("WRONGTYPE".to_string());
        assert!(!val.is_nil());
        assert!(val.is_error());
        assert_eq!(val.as_str(), None);
        assert_eq!(val.as_i64(), None);
        assert_eq!(val.as_array(), None);
        assert_eq!(val.to_display_string(), "(error) WRONGTYPE");
    }

    #[test]
    fn test_redis_value_status() {
        let val = RedisValue::Status("OK".to_string());
        assert_eq!(val.as_str(), Some("OK"));
        assert_eq!(val.to_display_string(), "OK");
    }

    #[test]
    fn test_redis_value_float() {
        let val = RedisValue::Float(3.14);
        assert_eq!(val.to_display_string(), "3.14");
    }

    #[test]
    fn test_redis_value_bool() {
        let val = RedisValue::Bool(true);
        assert_eq!(val.to_display_string(), "true");
    }

    #[test]
    fn test_redis_value_serde_roundtrip() {
        let values = vec![
            RedisValue::Nil,
            RedisValue::String("hello".to_string()),
            RedisValue::Integer(42),
            RedisValue::Float(2.5),
            RedisValue::Bool(false),
            RedisValue::Status("OK".to_string()),
            RedisValue::Error("ERR".to_string()),
            RedisValue::Array(vec![
                RedisValue::String("nested".to_string()),
                RedisValue::Nil,
            ]),
        ];

        for val in values {
            let json = serde_json::to_string(&val).unwrap();
            let deserialized: RedisValue = serde_json::from_str(&json).unwrap();
            assert_eq!(val, deserialized);
        }
    }
}
