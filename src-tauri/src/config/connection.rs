use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Standalone,
    Cluster,
    Sentinel,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Standalone
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SshAuth {
    KeyFile { path: String, passphrase: Option<String> },
    Password(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SslConfig {
    #[serde(default)]
    pub ca_cert: Option<PathBuf>,
    #[serde(default)]
    pub client_cert: Option<PathBuf>,
    #[serde(default)]
    pub client_key: Option<PathBuf>,
    #[serde(default)]
    pub skip_verify: bool,
}

impl Default for SslConfig {
    fn default() -> Self {
        Self {
            ca_cert: None,
            client_cert: None,
            client_key: None,
            skip_verify: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub db: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(rename = "type", default)]
    pub connection_type: ConnectionType,
    #[serde(default = "default_key_separator")]
    pub key_separator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<SshConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl: Option<SslConfig>,
    #[serde(default)]
    pub readonly: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub folder: Option<String>,
    #[serde(default)]
    pub use_ssl: bool,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_timeout() -> u64 {
    30
}

fn default_port() -> u16 {
    6379
}

fn default_key_separator() -> String {
    ":".to_string()
}

impl ConnectionConfig {
    pub fn new(name: impl Into<String>, host: impl Into<String>, port: u16) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            host: host.into(),
            port,
            db: 0,
            password: None,
            username: None,
            connection_type: ConnectionType::default(),
            key_separator: default_key_separator(),
            ssh: None,
            ssl: None,
            readonly: false,
            folder: None,
            use_ssl: false,
            timeout: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_connection_config() {
        let config = ConnectionConfig::new("local", "127.0.0.1", 6379);
        assert_eq!(config.name, "local");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 6379);
        assert_eq!(config.db, 0);
        assert_eq!(config.password, None);
        assert_eq!(config.username, None);
        assert_eq!(config.connection_type, ConnectionType::Standalone);
        assert_eq!(config.key_separator, ":");
        assert_eq!(config.ssh, None);
        assert_eq!(config.ssl, None);
        assert!(!config.readonly);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let config = ConnectionConfig::new("prod", "redis.example.com", 6380);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_ssh_config() {
        let ssh = SshConfig {
            host: "bastion.example.com".to_string(),
            port: 22,
            username: "admin".to_string(),
            auth: SshAuth::KeyFile("/home/admin/.ssh/id_rsa".to_string()),
        };
        let mut config = ConnectionConfig::new("tunnel", "127.0.0.1", 6379);
        config.ssh = Some(ssh.clone());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ssh, Some(ssh));
    }

    #[test]
    fn test_ssl_config() {
        let ssl = SslConfig {
            ca_cert: Some(PathBuf::from("/etc/ssl/ca.pem")),
            client_cert: Some(PathBuf::from("/etc/ssl/client.pem")),
            client_key: Some(PathBuf::from("/etc/ssl/client.key")),
            skip_verify: false,
        };
        let mut config = ConnectionConfig::new("secure", "redis.example.com", 6380);
        config.ssl = Some(ssl.clone());

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ssl, Some(ssl));
    }

    #[test]
    fn test_connection_type_variants() {
        let types = [
            ConnectionType::Standalone,
            ConnectionType::Cluster,
            ConnectionType::Sentinel,
        ];

        for ct in &types {
            let json = serde_json::to_string(ct).unwrap();
            let deserialized: ConnectionType = serde_json::from_str(&json).unwrap();
            assert_eq!(*ct, deserialized);
        }

        assert_eq!(serde_json::to_string(&ConnectionType::Standalone).unwrap(), "\"standalone\"");
        assert_eq!(serde_json::to_string(&ConnectionType::Cluster).unwrap(), "\"cluster\"");
        assert_eq!(serde_json::to_string(&ConnectionType::Sentinel).unwrap(), "\"sentinel\"");
    }

    #[test]
    fn test_connection_config_with_all_fields() {
        let config = ConnectionConfig {
            id: Uuid::new_v4(),
            name: "full".to_string(),
            host: "redis.example.com".to_string(),
            port: 6380,
            db: 3,
            password: Some("secret".to_string()),
            username: Some("admin".to_string()),
            connection_type: ConnectionType::Cluster,
            key_separator: "::".to_string(),
            ssh: Some(SshConfig {
                host: "bastion.example.com".to_string(),
                port: 2222,
                username: "ec2-user".to_string(),
                auth: SshAuth::Password("sshpass".to_string()),
            }),
            ssl: Some(SslConfig {
                ca_cert: Some(PathBuf::from("/etc/ssl/ca.pem")),
                client_cert: None,
                client_key: None,
                skip_verify: true,
            }),
            readonly: true,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
        assert_eq!(deserialized.password, Some("secret".to_string()));
        assert_eq!(deserialized.username, Some("admin".to_string()));
        assert!(deserialized.ssh.is_some());
        assert!(deserialized.ssl.is_some());
        assert!(deserialized.readonly);
    }

    #[test]
    fn test_connection_type_serialization() {
        let config = ConnectionConfig::new("test", "localhost", 6379);
        let json = serde_json::to_value(&config).unwrap();
        assert!(json.get("type").is_some(), "JSON must have 'type' field");
        assert_eq!(json["type"], "standalone");

        let mut cluster_config = ConnectionConfig::new("cluster", "localhost", 6379);
        cluster_config.connection_type = ConnectionType::Cluster;
        let json = serde_json::to_value(&cluster_config).unwrap();
        assert_eq!(json["type"], "cluster");

        let mut sentinel_config = ConnectionConfig::new("sentinel", "localhost", 6379);
        sentinel_config.connection_type = ConnectionType::Sentinel;
        let json = serde_json::to_value(&sentinel_config).unwrap();
        assert_eq!(json["type"], "sentinel");
    }
}
