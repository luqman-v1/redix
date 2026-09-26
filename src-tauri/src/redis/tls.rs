use crate::config::connection::ConnectionConfig;
use redis::{ClientTlsConfig, TlsCertificates};
use std::fs;

pub fn build_tls_certificates(config: &ConnectionConfig) -> Result<TlsCertificates, String> {
    let ssl = match &config.ssl {
        Some(s) => s,
        None => {
            return Ok(TlsCertificates {
                client_tls: None,
                root_cert: None,
            })
        }
    };

    let root_cert = if let Some(ca_path) = &ssl.ca_cert {
        Some(fs::read(ca_path).map_err(|e| format!("Failed to read CA cert: {}", e))?)
    } else {
        None
    };

    let client_tls = if let (Some(cert_path), Some(key_path)) = (&ssl.client_cert, &ssl.client_key)
    {
        let client_cert =
            fs::read(cert_path).map_err(|e| format!("Failed to read client cert: {}", e))?;
        let client_key =
            fs::read(key_path).map_err(|e| format!("Failed to read client key: {}", e))?;
        Some(ClientTlsConfig {
            client_cert,
            client_key,
        })
    } else {
        None
    };

    Ok(TlsCertificates {
        client_tls,
        root_cert,
    })
}
