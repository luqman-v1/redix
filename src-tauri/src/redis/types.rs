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
