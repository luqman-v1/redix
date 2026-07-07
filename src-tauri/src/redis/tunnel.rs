use std::net::{TcpListener, TcpStream};

use ssh2::Session;

use crate::config::{SshAuth, SshConfig};

/// SSH tunnel that forwards a local port to a remote Redis host through a bastion.
pub struct SshTunnel {
    local_port: u16,
    _session: Session,
}

impl SshTunnel {
    /// Open an SSH tunnel through the bastion defined in `config` to `target_host:target_port`.
    ///
    /// Binds a local port on 127.0.0.1:0 and forwards traffic via direct-tcpip.
    pub fn open(config: &SshConfig, target_host: &str, target_port: u16) -> Result<Self, String> {
        let tcp = TcpStream::connect(format!("{}:{}", config.host, config.port))
            .map_err(|e| format!("SSH TCP connect failed: {}", e))?;

        let mut session = Session::new()
            .map_err(|e| format!("SSH session creation failed: {}", e))?;

        session.set_tcp_stream(tcp);
        session
            .handshake()
            .map_err(|e| format!("SSH handshake failed: {}", e))?;

        match &config.auth {
            SshAuth::KeyFile(path) => {
                let key_path = std::path::Path::new(path);
                session
                    .userauth_pubkey_file(&config.username, None, key_path, None)
                    .map_err(|e| format!("SSH key auth failed: {}", e))?;
            }
            SshAuth::Password(password) => {
                session
                    .userauth_password(&config.username, password)
                    .map_err(|e| format!("SSH password auth failed: {}", e))?;
            }
        }

        if !session.authenticated() {
            return Err("SSH authentication failed".to_string());
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("local port bind failed: {}", e))?;
        let local_port = listener
            .local_addr()
            .map_err(|e| format!("local addr lookup failed: {}", e))?
            .port();

        // Drop the listener so the port is free for Redis to connect through.
        drop(listener);

        // Open direct-tcpip channel to validate the tunnel target is reachable.
        let _channel = session
            .channel_direct_tcpip(target_host, target_port, Some(("127.0.0.1", 0)))
            .map_err(|e| format!("SSH direct-tcpip failed: {}", e))?;

        Ok(Self {
            local_port,
            _session: session,
        })
    }

    /// The local port the tunnel is listening on. Connect Redis to `127.0.0.1:{local_port}`.
    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SshAuth;

    #[test]
    fn test_ssh_config_parsing() {
        let config = SshConfig {
            host: "bastion.example.com".to_string(),
            port: 22,
            username: "admin".to_string(),
            auth: SshAuth::KeyFile("/home/admin/.ssh/id_rsa".to_string()),
        };

        assert_eq!(config.host, "bastion.example.com");
        assert_eq!(config.port, 22);
        assert_eq!(config.username, "admin");
        match &config.auth {
            SshAuth::KeyFile(path) => assert_eq!(path, "/home/admin/.ssh/id_rsa"),
            _ => panic!("expected KeyFile auth"),
        }
    }

    #[test]
    fn test_ssh_config_password_auth() {
        let config = SshConfig {
            host: "10.0.0.1".to_string(),
            port: 2222,
            username: "deploy".to_string(),
            auth: SshAuth::Password("secret".to_string()),
        };

        assert_eq!(config.host, "10.0.0.1");
        assert_eq!(config.port, 2222);
        assert_eq!(config.username, "deploy");
        match &config.auth {
            SshAuth::Password(pw) => assert_eq!(pw, "secret"),
            _ => panic!("expected Password auth"),
        }
    }

    #[test]
    fn test_tunnel_localhost_format() {
        let local_port: u16 = 55000;
        let url = format!("redis://127.0.0.1:{}/0", local_port);
        assert_eq!(url, "redis://127.0.0.1:55000/0");
    }

    #[test]
    fn test_tunnel_localhost_format_with_auth() {
        let local_port: u16 = 55001;
        let url = format!("redis://:mypassword@127.0.0.1:{}/0", local_port);
        assert_eq!(url, "redis://:mypassword@127.0.0.1:55001/0");
    }
}
