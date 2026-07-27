pub mod client;
pub mod cluster;
pub mod sentinel;
pub mod standalone;
pub mod teleport;
pub mod tls;
pub mod tunnel;
pub mod types;

pub use client::*;
pub use cluster::*;
pub use sentinel::*;
pub use standalone::*;
pub use teleport::*;
pub use tunnel::*;
pub use types::*;

pub fn emit_command_log(cmd: &str, duration_ms: u64) {
    use tauri::Emitter;
    if let Some(app) = crate::APP_HANDLE.get() {
        let payload = serde_json::json!({
            "command": cmd,
            "duration": duration_ms
        });
        let _ = app.emit("command-log", payload);
    }
}

pub const UNSAFE_CMDS: &[&str] = &[
    "SET", "DEL", "FLUSHALL", "FLUSHDB", "RENAME", "RENAMENX", "HSET", "HSETNX", 
    "RPUSH", "LPUSH", "LSET", "SADD", "ZADD", "HDEL", "SREM", "ZREM", "LPOP", "RPOP", "PERSIST", 
    "INCR", "DECR", "MSET", "XADD", "GEOADD", "LREM", "SPOP", "GETSET", "APPEND", 
    "SETEX", "SETNX", "PSETEX", "SETRANGE", "MSETNX", "EXPIRE", "CONFIG",
    "EVAL", "EVALSHA", "FUNCTION", "SCRIPT", "DEBUG"
];
