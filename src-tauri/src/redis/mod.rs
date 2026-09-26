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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionConfig;

    #[test]
    fn read_only_rejects_writes_but_allows_reads() {
        assert!(ensure_writable(true, "SET").is_err());
        assert!(ensure_writable(true, "del").is_err());
        assert!(ensure_writable(true, "GET").is_ok());
        assert!(ensure_writable(true, "SCAN 0 COUNT 10").is_ok());
    }

    #[test]
    fn read_only_never_rejects_when_writable() {
        assert!(ensure_writable(false, "SET").is_ok());
        assert!(ensure_writable(false, "FLUSHALL").is_ok());
    }

    #[test]
    fn read_only_pipeline_rejects_if_any_command_writes() {
        let safe = vec![
            ("TTL".to_string(), vec!["a".to_string()]),
            ("TYPE".to_string(), vec!["a".to_string()]),
        ];
        assert!(ensure_pipeline_writable(true, &safe).is_ok());

        let unsafe_batch = vec![
            ("TTL".to_string(), vec!["a".to_string()]),
            ("DEL".to_string(), vec!["a".to_string()]),
        ];
        assert!(ensure_pipeline_writable(true, &unsafe_batch).is_err());
        assert!(ensure_pipeline_writable(false, &unsafe_batch).is_ok());
    }

    #[test]
    fn quiet_commands_are_filtered_case_insensitively() {
        assert!(is_quiet_command("PING"));
        assert!(is_quiet_command("scan 0 count 100"));
        assert!(is_quiet_command("PIPELINE 500 cmds"));
        assert!(!is_quiet_command("GET mykey"));
        assert!(!is_quiet_command("SET mykey value"));
    }

    #[test]
    fn url_without_credentials_has_no_auth_segment() {
        let config = ConnectionConfig::new("plain", "127.0.0.1", 6379);
        assert_eq!(build_redis_url(&config, true), "redis://127.0.0.1:6379/0");
        assert_eq!(build_redis_url(&config, false), "redis://127.0.0.1:6379");
    }

    #[test]
    fn url_percent_encodes_credentials_with_reserved_characters() {
        let mut config = ConnectionConfig::new("creds", "127.0.0.1", 6379);
        config.username = Some("user@corp".to_string());
        config.password = Some("p@ss:w/rd".to_string());

        let url = build_redis_url(&config, true);
        assert_eq!(url, "redis://user%40corp:p%40ss%3Aw%2Frd@127.0.0.1:6379/0");
        // The raw credential must not leak into the authority unescaped.
        assert!(!url.contains("p@ss"));
    }

    #[test]
    fn url_uses_password_only_when_username_absent() {
        let mut config = ConnectionConfig::new("pw", "127.0.0.1", 6379);
        config.password = Some("secret".to_string());
        assert_eq!(build_redis_url(&config, true), "redis://:secret@127.0.0.1:6379/0");
    }

    #[test]
    fn url_switches_scheme_and_appends_insecure_for_tls() {
        let mut config = ConnectionConfig::new("tls", "example.com", 6380);
        config.use_ssl = true;
        assert_eq!(build_redis_url(&config, false), "rediss://example.com:6380");

        config.ssl = Some(crate::config::SslConfig {
            ca_cert: None,
            client_cert: None,
            client_key: None,
            skip_verify: true,
        });
        assert_eq!(
            build_redis_url(&config, false),
            "rediss://example.com:6380#insecure"
        );
    }
}
