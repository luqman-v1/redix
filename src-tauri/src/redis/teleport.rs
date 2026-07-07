use crate::redis::client::RedisClient;

/// Commands blocked by Teleport Redis proxy.
const RESTRICTED: &[&str] = &[
    "CLUSTER",
    "CONFIG",
    "DEBUG",
    "SCRIPT",
    "SHUTDOWN",
    "SLAVEOF",
    "REPLICAOF",
    "CLIENT",
    "ACL",
    "MODULE",
    "SWAPDB",
    "LATENCY",
    "MEMORY",
    "MONITOR",
    "KEYS",
    "SAVE",
    "BGSAVE",
    "BGREWRITEAOF",
    "LASTSAVE",
    "DBSIZE",
    "FLUSHDB",
    "FLUSHALL",
    "SORT",
];

/// Returns the list of Teleport-restricted commands as owned Strings.
pub fn restricted_commands() -> Vec<String> {
    RESTRICTED.iter().map(|s| s.to_string()).collect()
}

/// Check if a command string starts with a Teleport-restricted command (case-insensitive).
pub fn is_restricted(command: &str) -> bool {
    let first_word = command.split_whitespace().next().unwrap_or("");
    RESTRICTED
        .iter()
        .any(|r| r.eq_ignore_ascii_case(first_word))
}

/// Detect whether the Redis connection is behind Teleport by probing `COMMAND INFO CLUSTER`.
/// Returns `true` if Teleport is detected (command returns an error).
pub async fn detect_teleport(client: &dyn RedisClient) -> bool {
    client
        .execute("COMMAND", vec!["INFO".into(), "CLUSTER".into()])
        .await
        .is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restricted_commands_list() {
        let cmds = restricted_commands();
        assert!(cmds.contains(&"CLUSTER".to_string()));
        assert!(cmds.contains(&"CONFIG".to_string()));
        assert!(cmds.contains(&"DEBUG".to_string()));
        assert!(cmds.contains(&"KEYS".to_string()));
        assert!(!cmds.contains(&"GET".to_string()));
    }

    #[test]
    fn test_is_restricted() {
        assert!(is_restricted("CLUSTER INFO"));
        assert!(is_restricted("CONFIG SET"));
        assert!(!is_restricted("GET mykey"));
        assert!(!is_restricted("SET mykey value"));
    }
}
