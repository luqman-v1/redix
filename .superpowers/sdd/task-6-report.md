# Task 6 Report: Encrypted Connection Store

## Status: COMPLETE

## Changes Made

### Dependencies Added (`src-tauri/Cargo.toml`)
- `aes-gcm = "0.10"` - AES-256-GCM encryption
- `pbkdf2 = "0.12"` with `hmac` feature - key derivation
- `sha2 = "0.10"` - SHA-256 for PBKDF2
- `rand = "0.8"` - secure random nonce generation
- `whoami = "1"` - hostname for key derivation
- `dirs = "5"` - config directory resolution
- `tempfile = "3"` (dev-dependencies)

### New File: `src-tauri/src/config/store.rs`
- `ConnectionStore` struct: `path: PathBuf`, `cipher: Aes256Gcm`
- `derive_key()`: PBKDF2-HMAC-SHA256 with 100k iterations, salt `redix-store-salt`, password `redix-{hostname}`
- `ConnectionStore::new(config_dir)`: creates dir, derives key, builds cipher
- `ConnectionStore::default()`: uses `dirs::config_dir().join("redix")`
- `load()`: reads file, splits nonce (12 bytes) from ciphertext, decrypts, deserializes
- `save(connections)`: serializes, generates random nonce, encrypts, writes nonce+ciphertext
- `add(config)`: load -> push -> save
- `update(config)`: load -> find by id -> replace -> save
- `delete(id)`: load -> retain (remove matching id) -> save

### Modified: `src-tauri/src/config/mod.rs`
- Added `pub mod store; pub use store::*;`

## Tests (5 tests, all pass)
1. `test_save_and_load_empty` - empty vec roundtrip
2. `test_add_and_load` - add single config, verify load
3. `test_update_existing` - update name/host, verify persistence
4. `test_delete_by_id` - delete one of two, verify correct one removed
5. `test_file_is_encrypted` - raw file bytes don't contain "secret" or "localhost"

## Commit
`f42c3b8` - `feat: encrypted connection store with AES-256-GCM`

## TDD Notes
Tests written first, verified failing (missing store module), then implementation written to pass.
