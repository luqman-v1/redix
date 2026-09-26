pub mod client;
pub mod cluster;
pub mod sentinel;
pub mod standalone;
pub mod teleport;
pub mod tls;
pub mod types;

pub use client::*;
pub use cluster::*;
pub use sentinel::*;
pub use standalone::*;
pub use teleport::*;
pub use types::*;

const QUIET_COMMANDS: &[&str] = &["PING", "SCAN", "TYPE", "TTL", "PTTL", "INFO", "PIPELINE"];

pub const UNSAFE_CMDS: &[&str] = &[
    "SET", "DEL", "FLUSHALL", "FLUSHDB", "RENAME", "RENAMENX", "HSET", "HSETNX", "RPUSH", "LPUSH",
    "LSET", "SADD", "ZADD", "HDEL", "SREM", "ZREM", "LPOP", "RPOP", "PERSIST", "INCR", "DECR",
    "MSET", "XADD", "GEOADD", "LREM", "SPOP", "GETSET", "APPEND", "SETEX", "SETNX", "PSETEX",
    "SETRANGE", "MSETNX", "EXPIRE", "CONFIG", "EVAL", "EVALSHA", "FUNCTION", "SCRIPT", "DEBUG",
];

pub fn is_quiet_command(cmd: &str) -> bool {
    let first = cmd.split_whitespace().next().unwrap_or(cmd);
    QUIET_COMMANDS.iter().any(|q| q.eq_ignore_ascii_case(first))
}

pub fn emit_command_log(cmd: &str, duration_ms: u64) {
    if is_quiet_command(cmd) {
        return;
    }
    use tauri::Emitter;
    if let Some(app) = crate::APP_HANDLE.get() {
        let payload = serde_json::json!({
            "command": cmd,
            "duration": duration_ms
        });
        let _ = app.emit("command-log", payload);
    }
}

pub fn ensure_writable(readonly: bool, cmd: &str) -> Result<(), String> {
    if readonly {
        let upper = cmd.to_ascii_uppercase();
        if UNSAFE_CMDS.contains(&upper.as_str()) {
            return Err("Connection is in Read-Only mode".to_string());
        }
    }
    Ok(())
}

pub fn ensure_pipeline_writable(
    readonly: bool,
    cmds: &[(String, Vec<String>)],
) -> Result<(), String> {
    if readonly {
        for (cmd, _) in cmds {
            ensure_writable(true, cmd)?;
        }
    }
    Ok(())
}

/// Shared Redis URL builder. Percent-encodes credentials so special
/// characters (`@`, `:`, `/`) don't corrupt the connection URL.
pub fn build_redis_url(config: &crate::config::ConnectionConfig, include_db: bool) -> String {
    use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
    let use_ssl = config.use_ssl || config.ssl.is_some();
    let scheme = if use_ssl { "rediss://" } else { "redis://" };
    let mut url = String::from(scheme);
    let encode = |s: &str| utf8_percent_encode(s, NON_ALPHANUMERIC).to_string();
    match (&config.username, &config.password) {
        (Some(user), Some(pass)) => url.push_str(&format!("{}:{}@", encode(user), encode(pass))),
        (None, Some(pass)) => url.push_str(&format!(":{}@", encode(pass))),
        _ => {}
    }
    if include_db {
        url.push_str(&format!("{}:{}/{}", config.host, config.port, config.db));
    } else {
        url.push_str(&format!("{}:{}", config.host, config.port));
    }
    if let Some(ssl) = &config.ssl {
        if ssl.skip_verify {
            url.push_str("#insecure");
        }
    }
    url
}
