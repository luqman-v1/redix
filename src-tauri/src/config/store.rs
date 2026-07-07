use std::fs;
use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

use crate::config::connection::ConnectionConfig;

const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;
const FILE_NAME: &str = "connections.enc";

fn derive_key() -> Vec<u8> {
    let password = format!("redix-{}", whoami::hostname());
    let salt = b"redix-store-salt";
    let mut key = vec![0u8; KEY_LEN];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub struct ConnectionStore {
    path: PathBuf,
    cipher: Aes256Gcm,
}

impl ConnectionStore {
    pub fn new(config_dir: &Path) -> Self {
        fs::create_dir_all(config_dir).expect("failed to create config directory");
        let key = derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key).expect("failed to create cipher");
        Self {
            path: config_dir.join(FILE_NAME),
            cipher,
        }
    }

    pub fn load(&self) -> Result<Vec<ConnectionConfig>, String> {
        let data = match fs::read(&self.path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
            Err(e) => return Err(format!("failed to read store: {}", e)),
        };
        if data.len() < NONCE_LEN {
            return Err("store file too short".into());
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("decryption failed: {}", e))?;
        serde_json::from_slice(&plaintext).map_err(|e| format!("deserialization failed: {}", e))
    }

    pub fn save(&self, connections: &[ConnectionConfig]) -> Result<(), String> {
        let plaintext =
            serde_json::to_vec(connections).map_err(|e| format!("serialization failed: {}", e))?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| format!("encryption failed: {}", e))?;
        let mut output = nonce_bytes.to_vec();
        output.extend_from_slice(&ciphertext);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create dir: {}", e))?;
        }
        fs::write(&self.path, output).map_err(|e| format!("failed to write store: {}", e))
    }

    pub fn add(&self, config: ConnectionConfig) -> Result<(), String> {
        let mut connections = self.load()?;
        connections.push(config);
        self.save(&connections)
    }

    pub fn update(&self, config: ConnectionConfig) -> Result<(), String> {
        let mut connections = self.load()?;
        if let Some(existing) = connections.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
            self.save(&connections)
        } else {
            Err(format!("connection not found: {}", config.id))
        }
    }

    pub fn delete(&self, id: uuid::Uuid) -> Result<(), String> {
        let mut connections = self.load()?;
        let before = connections.len();
        connections.retain(|c| c.id != id);
        if connections.len() == before {
            return Err(format!("connection not found: {}", id));
        }
        self.save(&connections)
    }
}

impl Default for ConnectionStore {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .expect("failed to get config dir")
            .join("redix");
        Self::new(&config_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::connection::ConnectionConfig;

    fn temp_store() -> (ConnectionStore, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let store = ConnectionStore::new(dir.path());
        (store, dir)
    }

    #[test]
    fn test_save_and_load_empty() {
        let (store, _dir) = temp_store();
        store.save(&[]).unwrap();
        let loaded = store.load().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_add_and_load() {
        let (store, _dir) = temp_store();
        let config = ConnectionConfig::new("test", "localhost", 6379);
        let id = config.id;
        store.add(config).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id);
        assert_eq!(loaded[0].name, "test");
        assert_eq!(loaded[0].host, "localhost");
    }

    #[test]
    fn test_update_existing() {
        let (store, _dir) = temp_store();
        let mut config = ConnectionConfig::new("original", "localhost", 6379);
        let id = config.id;
        store.add(config.clone()).unwrap();

        config.name = "updated".to_string();
        config.host = "10.0.0.1".to_string();
        store.update(config).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id);
        assert_eq!(loaded[0].name, "updated");
        assert_eq!(loaded[0].host, "10.0.0.1");
    }

    #[test]
    fn test_delete_by_id() {
        let (store, _dir) = temp_store();
        let config1 = ConnectionConfig::new("first", "localhost", 6379);
        let config2 = ConnectionConfig::new("second", "localhost", 6380);
        let id1 = config1.id;
        store.add(config1).unwrap();
        store.add(config2).unwrap();

        store.delete(id1).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "second");
    }

    #[test]
    fn test_file_is_encrypted() {
        let (store, dir) = temp_store();
        let config = ConnectionConfig::new("secret", "localhost", 6379);
        store.add(config).unwrap();

        let raw = fs::read(dir.path().join(FILE_NAME)).unwrap();
        let raw_str = String::from_utf8_lossy(&raw);
        assert!(
            !raw_str.contains("secret"),
            "file should not contain plaintext name"
        );
        assert!(
            !raw_str.contains("localhost"),
            "file should not contain plaintext host"
        );
    }

    #[test]
    fn test_update_nonexistent_returns_error() {
        let (store, _dir) = temp_store();
        let config = ConnectionConfig::new("ghost", "localhost", 6379);
        let result = store.update(config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("connection not found"));
    }

    #[test]
    fn test_delete_nonexistent_is_noop() {
        let (store, _dir) = temp_store();
        let config = ConnectionConfig::new("exists", "localhost", 6379);
        store.add(config).unwrap();

        let fake_id = uuid::Uuid::new_v4();
        let result = store.delete(fake_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("connection not found"));

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "exists");
    }

    #[test]
    fn test_multiple_connections_order_preserved() {
        let (store, _dir) = temp_store();
        let names = ["alpha", "beta", "gamma", "delta"];
        for name in &names {
            store
                .add(ConnectionConfig::new(*name, "localhost", 6379))
                .unwrap();
        }

        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 4);
        for (i, name) in names.iter().enumerate() {
            assert_eq!(loaded[i].name, *name);
        }
    }
}
