use std::fs;
use std::path::{Path, PathBuf};

use log::{error, info};

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
    // ponytail: must match old whoami::hostname() behavior exactly (unwrap_or_default)
    // or existing encrypted connections.enc files become unreadable
    let password = format!("redix-{}", whoami::fallible::hostname().unwrap_or_default());
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
        info!("Loading connections from {:?}", self.path);
        let data = match fs::read(&self.path) {
            Ok(d) => d,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                info!("No connections file found, returning empty list");
                return Ok(vec![]);
            }
            Err(e) => {
                error!("Failed to read store file {:?}: {}", self.path, e);
                return Err(format!("failed to read store: {}", e));
            }
        };
        if data.len() < NONCE_LEN {
            error!("Store file too short ({} bytes), backing up and starting fresh", data.len());
            self.backup_corrupt_file();
            return Ok(vec![]);
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plaintext = match self.cipher.decrypt(nonce, ciphertext) {
            Ok(p) => p,
            Err(e) => {
                error!("Decryption failed for {:?}: {} (file size: {} bytes). Backing up corrupt file and starting fresh.", self.path, e, data.len());
                self.backup_corrupt_file();
                return Ok(vec![]);
            }
        };
        let result: Vec<ConnectionConfig> = match serde_json::from_slice(&plaintext) {
            Ok(r) => r,
            Err(e) => {
                error!("Deserialization failed: {}. Backing up corrupt file and starting fresh.", e);
                self.backup_corrupt_file();
                return Ok(vec![]);
            }
        };
        info!("Loaded {} connections", result.len());
        Ok(result)
    }

    fn backup_corrupt_file(&self) {
        let backup = self.path.with_extension("enc.bak");
        match fs::rename(&self.path, &backup) {
            Ok(_) => error!("Corrupt store backed up to {:?}", backup),
            Err(e) => error!("Failed to back up corrupt store: {}", e),
        }
    }

    pub fn save(&self, connections: &[ConnectionConfig]) -> Result<(), String> {
        info!("Saving {} connections to {:?}", connections.len(), self.path);
        let plaintext =
            serde_json::to_vec(connections).map_err(|e| {
                error!("Serialization failed: {}", e);
                format!("serialization failed: {}", e)
            })?;
        let mut nonce_bytes = [0u8; NONCE_LEN];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| {
                error!("Encryption failed: {}", e);
                format!("encryption failed: {}", e)
            })?;
        let mut output = nonce_bytes.to_vec();
        output.extend_from_slice(&ciphertext);
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("failed to create dir: {}", e))?;
        }
        fs::write(&self.path, &output).map_err(|e| {
            error!("Failed to write store: {}", e);
            format!("failed to write store: {}", e)
        })?;
        info!("Store saved successfully ({} bytes)", output.len());
        Ok(())
    }

    pub fn add(&self, config: ConnectionConfig) -> Result<(), String> {
        info!("Adding connection '{}' ({})", config.name, config.id);
        let mut connections = self.load()?;
        connections.push(config);
        self.save(&connections)
    }

    pub fn update(&self, config: ConnectionConfig) -> Result<(), String> {
        info!("Updating connection '{}' ({})", config.name, config.id);
        let mut connections = self.load()?;
        if let Some(existing) = connections.iter_mut().find(|c| c.id == config.id) {
            *existing = config;
            self.save(&connections)
        } else {
            error!("Connection not found for update: {}", config.id);
            Err(format!("connection not found: {}", config.id))
        }
    }

    pub fn delete(&self, id: uuid::Uuid) -> Result<(), String> {
        info!("Deleting connection {}", id);
        let mut connections = self.load()?;
        let before = connections.len();
        connections.retain(|c| c.id != id);
        if connections.len() == before {
            error!("Connection not found for delete: {}", id);
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
