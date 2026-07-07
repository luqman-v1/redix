# Phase 1: Connection Management

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Full connection CRUD with encrypted storage, multi-connection switching, and visual status indicators.

**Architecture:** Rust backend handles config model + encryption. Frontend manages UI. Tauri commands bridge them.

**Tech Stack:** Tauri v2 commands, serde, AES-256-GCM, Svelte stores

## Global Constraints
- Node.js 20+, Rust stable (1.77+)
- pnpm, no mutation outside stores
- Files < 800 lines, functions < 50 lines

---

## Task 5: Connection Config Model (Rust)

**Files:**
- Create: `src-tauri/src/config/connection.rs`
- Create: `src-tauri/src/config/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: `ConnectionConfig`, `ConnectionType`, `SshConfig`, `SshAuth`, `SslConfig` — all serde-serializable

**Steps:**

- [ ] **Step 1: Write tests for ConnectionConfig**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_connection_config() {
        let config = ConnectionConfig::new("test-redis", "localhost", 6379);
        assert_eq!(config.name, "test-redis");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert_eq!(config.db, 0);
        assert_eq!(config.key_separator, ":");
        assert_eq!(config.connection_type, ConnectionType::Standalone);
        assert!(!config.readonly);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let config = ConnectionConfig::new("test", "127.0.0.1", 6380);
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_ssh_config() {
        let mut config = ConnectionConfig::new("ssh-redis", "localhost", 6379);
        config.ssh = Some(SshConfig {
            host: "bastion.example.com".into(),
            port: 22,
            username: "admin".into(),
            auth: SshAuth::KeyFile("/home/user/.ssh/id_rsa".into()),
        });
        assert!(config.ssh.is_some());
    }

    #[test]
    fn test_ssl_config() {
        let mut config = ConnectionConfig::new("ssl-redis", "localhost", 6379);
        config.ssl = Some(SslConfig {
            ca_cert: Some("/path/to/ca.pem".into()),
            client_cert: None,
            client_key: None,
            skip_verify: false,
        });
        assert!(!config.ssl.unwrap().skip_verify);
    }

    #[test]
    fn test_connection_type_variants() {
        assert_ne!(ConnectionType::Standalone, ConnectionType::Cluster);
        assert_ne!(ConnectionType::Cluster, ConnectionType::Sentinel);
    }
}
```

- [ ] **Step 2: Run tests, verify fail**

```bash
cd src-tauri && cargo test
```

Expected: Compilation errors.

- [ ] **Step 3: Implement ConnectionConfig**

```rust
// src-tauri/src/config/connection.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionType {
    Standalone,
    Cluster,
    Sentinel,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SshAuth {
    KeyFile(String),
    Password(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SslConfig {
    pub ca_cert: Option<PathBuf>,
    pub client_cert: Option<PathBuf>,
    pub client_key: Option<PathBuf>,
    #[serde(default)]
    pub skip_verify: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub db: i64,
    pub password: Option<String>,
    pub username: Option<String>,
    #[serde(default, rename = "type")]
    pub connection_type: ConnectionType,
    #[serde(default = "default_separator")]
    pub key_separator: String,
    pub ssh: Option<SshConfig>,
    pub ssl: Option<SslConfig>,
    #[serde(default)]
    pub readonly: bool,
}

fn default_port() -> u16 { 6379 }
fn default_separator() -> String { ":".to_string() }

impl ConnectionConfig {
    pub fn new(name: &str, host: &str, port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            host: host.to_string(),
            port,
            db: 0,
            password: None,
            username: None,
            connection_type: ConnectionType::Standalone,
            key_separator: ":".to_string(),
            ssh: None,
            ssl: None,
            readonly: false,
        }
    }
}
```

- [ ] **Step 4: Add Cargo.toml deps**

```toml
uuid = { version = "1", features = ["v4", "serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

- [ ] **Step 5: Create mod.rs + register in main.rs**

```rust
// src-tauri/src/config/mod.rs
pub mod connection;
pub use connection::*;
```

```rust
// src-tauri/src/main.rs
mod config;

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Run tests, verify pass**

```bash
cd src-tauri && cargo test
```

Expected: All 5 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add . && git commit -m "feat: connection config model with tests"
```

---

## Task 6: Encrypted Credential Storage (Rust)

**Files:**
- Create: `src-tauri/src/config/store.rs`
- Modify: `src-tauri/src/config/mod.rs`

**Interfaces:**
- Produces: `ConnectionStore` with `load()`, `save()`, `add()`, `update()`, `delete()`

**Steps:**

- [ ] **Step 1: Write tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_store() -> (ConnectionStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = ConnectionStore::new(dir.path()).unwrap();
        (store, dir)
    }

    #[test]
    fn test_save_and_load_empty() {
        let (store, _dir) = test_store();
        store.save(&[]).unwrap();
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn test_add_and_load() {
        let (store, _dir) = test_store();
        let config = ConnectionConfig::new("test", "localhost", 6379);
        let id = config.id.clone();
        store.add(&config).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, id);
    }

    #[test]
    fn test_update_existing() {
        let (store, _dir) = test_store();
        let mut config = ConnectionConfig::new("test", "localhost", 6379);
        store.add(&config).unwrap();
        config.name = "updated".to_string();
        store.update(&config).unwrap();
        assert_eq!(store.load().unwrap()[0].name, "updated");
    }

    #[test]
    fn test_delete_by_id() {
        let (store, _dir) = test_store();
        let config = ConnectionConfig::new("test", "localhost", 6379);
        let id = config.id.clone();
        store.add(&config).unwrap();
        store.delete(&id).unwrap();
        assert!(store.load().unwrap().is_empty());
    }

    #[test]
    fn test_file_is_encrypted() {
        let (store, dir) = test_store();
        store.add(&ConnectionConfig::new("secret", "localhost", 6379)).unwrap();
        let raw = std::fs::read_to_string(dir.path().join("connections.enc")).unwrap();
        assert!(!raw.contains("secret"));
        assert!(!raw.contains("localhost"));
    }
}
```

- [ ] **Step 2: Run tests, verify fail**

```bash
cd src-tauri && cargo test
```

Expected: `ConnectionStore` not found.

- [ ] **Step 3: Implement**

```rust
// src-tauri/src/config/store.rs
use super::ConnectionConfig;
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use std::fs;
use std::path::{Path, PathBuf};

const PBKDF2_ITERATIONS: u32 = 100_000;
const SALT: &[u8] = b"redix-connection-store-v1";

pub struct ConnectionStore {
    path: PathBuf,
    cipher: Aes256Gcm,
}

impl Default for ConnectionStore {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("redix");
        Self::new(&config_dir).expect("failed to create connection store")
    }
}

impl ConnectionStore {
    pub fn new(config_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        fs::create_dir_all(config_dir)?;
        let key = derive_key();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| format!("cipher init: {e}"))?;
        Ok(Self { path: config_dir.join("connections.enc"), cipher })
    }

    pub fn load(&self) -> Result<Vec<ConnectionConfig>, Box<dyn std::error::Error>> {
        if !self.path.exists() { return Ok(Vec::new()); }
        let encrypted = fs::read(&self.path)?;
        if encrypted.is_empty() { return Ok(Vec::new()); }
        let plaintext = self.decrypt(&encrypted)?;
        Ok(serde_json::from_slice(&plaintext)?)
    }

    pub fn save(&self, connections: &[ConnectionConfig]) -> Result<(), Box<dyn std::error::Error>> {
        let plaintext = serde_json::to_vec(connections)?;
        let encrypted = self.encrypt(&plaintext)?;
        fs::write(&self.path, encrypted)?;
        Ok(())
    }

    pub fn add(&self, config: &ConnectionConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut list = self.load()?;
        list.push(config.clone());
        self.save(&list)
    }

    pub fn update(&self, config: &ConnectionConfig) -> Result<(), Box<dyn std::error::Error>> {
        let mut list = self.load()?;
        match list.iter_mut().find(|c| c.id == config.id) {
            Some(existing) => { *existing = config.clone(); self.save(&list) }
            None => Err("Connection not found".into()),
        }
    }

    pub fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut list = self.load()?;
        list.retain(|c| c.id != id);
        self.save(&list)
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ct = self.cipher.encrypt(nonce, data)
            .map_err(|e| format!("encrypt: {e}"))?;
        let mut out = nonce_bytes.to_vec();
        out.extend_from_slice(&ct);
        Ok(out)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if data.len() < 12 { return Err("Invalid data".into()); }
        let (nonce_bytes, ct) = data.split_at(12);
        self.cipher.decrypt(Nonce::from_slice(nonce_bytes), ct)
            .map_err(|e| format!("decrypt: {e}").into())
    }
}

fn derive_key() -> Vec<u8> {
    let mut key = vec![0u8; 32];
    let password = format!("redix-{}", whoami::hostname());
    pbkdf2_hmac::<Sha256>(password.as_bytes(), SALT, PBKDF2_ITERATIONS, &mut key);
    key
}
```

- [ ] **Step 4: Add Cargo.toml deps**

```toml
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", features = ["hmac"] }
sha2 = "0.10"
rand = "0.8"
whoami = "1"
dirs = "5"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 5: Update mod.rs**

```rust
// src-tauri/src/config/mod.rs
pub mod connection;
pub mod store;
pub use connection::*;
pub use store::*;
```

- [ ] **Step 6: Run tests, verify pass**

```bash
cd src-tauri && cargo test
```

Expected: All tests PASS.

- [ ] **Step 7: Commit**

```bash
git add . && git commit -m "feat: encrypted connection store with AES-256-GCM"
```

---

## Task 7: Tauri Commands Bridge (Rust)

**Files:**
- Create: `src-tauri/src/commands/connections.rs`
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces: Tauri commands — `get_connections`, `add_connection`, `update_connection`, `delete_connection`, `test_connection`

**Steps:**

- [ ] **Step 1: Implement commands**

```rust
// src-tauri/src/commands/connections.rs
use crate::config::{ConnectionConfig, ConnectionStore};
use tauri::State;

#[tauri::command]
pub fn get_connections(store: State<ConnectionStore>) -> Result<Vec<ConnectionConfig>, String> {
    store.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_connection(store: State<ConnectionStore>, config: ConnectionConfig) -> Result<ConnectionConfig, String> {
    store.add(&config).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub fn update_connection(store: State<ConnectionStore>, config: ConnectionConfig) -> Result<ConnectionConfig, String> {
    store.update(&config).map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub fn delete_connection(store: State<ConnectionStore>, id: String) -> Result<(), String> {
    store.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<bool, String> {
    if config.host.is_empty() || config.port == 0 {
        return Err("Invalid host or port".to_string());
    }
    Ok(true)
}
```

- [ ] **Step 2: Create mod + register**

```rust
// src-tauri/src/commands/mod.rs
pub mod connections;
```

```rust
// src-tauri/src/main.rs
mod commands;
mod config;

fn main() {
    tauri::Builder::default()
        .manage(config::ConnectionStore::default())
        .invoke_handler(tauri::generate_handler![
            commands::connections::get_connections,
            commands::connections::add_connection,
            commands::connections::update_connection,
            commands::connections::delete_connection,
            commands::connections::test_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Build verify**

```bash
cd src-tauri && cargo build
```

Expected: Compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add . && git commit -m "feat: tauri commands for connection CRUD"
```

---

## Task 8: Connection Manager UI (Frontend)

**Files:**
- Create: `src/lib/types/connection.ts`
- Create: `src/lib/stores/connections.ts`
- Create: `src/lib/components/ConnectionList.svelte`
- Create: `src/lib/components/ConnectionForm.svelte`
- Modify: `src/routes/+page.svelte`

**Interfaces:**
- Produces: `connections` store, `activeConnection` store
- Produces: `<ConnectionList />`, `<ConnectionForm />`

**Steps:**

- [ ] **Step 1: Write TypeScript types**

```ts
// src/lib/types/connection.ts
export type ConnectionType = "standalone" | "cluster" | "sentinel";

export interface SshConfig {
  host: string;
  port: number;
  username: string;
  auth: { type: "keyfile"; path: string } | { type: "password"; password: string };
}

export interface SslConfig {
  ca_cert?: string;
  client_cert?: string;
  client_key?: string;
  skip_verify: boolean;
}

export interface ConnectionConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  db: number;
  password?: string;
  username?: string;
  type: ConnectionType;
  key_separator: string;
  ssh?: SshConfig;
  ssl?: SslConfig;
  readonly: boolean;
}
```

- [ ] **Step 2: Write connections store**

```ts
// src/lib/stores/connections.ts
import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { ConnectionConfig } from "$lib/types/connection";

function createConnectionsStore() {
  const { subscribe, set, update } = writable<ConnectionConfig[]>([]);

  return {
    subscribe,
    async load() {
      const list = await invoke<ConnectionConfig[]>("get_connections");
      set(list);
    },
    async add(config: ConnectionConfig) {
      const added = await invoke<ConnectionConfig>("add_connection", { config });
      update((list) => [...list, added]);
      return added;
    },
    async update(config: ConnectionConfig) {
      const updated = await invoke<ConnectionConfig>("update_connection", { config });
      update((list) => list.map((c) => (c.id === updated.id ? updated : c)));
      return updated;
    },
    async remove(id: string) {
      await invoke("delete_connection", { id });
      update((list) => list.filter((c) => c.id !== id));
    },
  };
}

export const connections = createConnectionsStore();
export const activeConnection = writable<ConnectionConfig | null>(null);
```

- [ ] **Step 3: Write ConnectionForm**

```svelte
<!-- src/lib/components/ConnectionForm.svelte -->
<script lang="ts">
  import type { ConnectionConfig, ConnectionType } from "$lib/types/connection";
  import { connections } from "$lib/stores/connections";

  let { editing = null, onclose }: { editing?: ConnectionConfig | null; onclose: () => void } = $props();

  let name = $state(editing?.name ?? "");
  let host = $state(editing?.host ?? "localhost");
  let port = $state(editing?.port ?? 6379);
  let db = $state(editing?.db ?? 0);
  let password = $state(editing?.password ?? "");
  let connectionType = $state<ConnectionType>(editing?.type ?? "standalone");

  async function handleSubmit(e: Event) {
    e.preventDefault();
    const config: ConnectionConfig = {
      id: editing?.id ?? crypto.randomUUID(),
      name, host, port, db,
      password: password || undefined,
      type: connectionType,
      key_separator: editing?.key_separator ?? ":",
      readonly: editing?.readonly ?? false,
    };
    if (editing) { await connections.update(config); }
    else { await connections.add(config); }
    onclose();
  }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center" style:background-color="rgba(0,0,0,0.5)">
  <form onsubmit={handleSubmit} class="rounded-lg p-6 w-96" style:background-color="var(--color-surface)">
    <h3 class="text-lg font-bold mb-4" style:color="var(--color-fg)">
      {editing ? "Edit Connection" : "New Connection"}
    </h3>
    <div class="space-y-3">
      <div>
        <label class="block text-sm mb-1" style:color="var(--color-muted)">Name</label>
        <input bind:value={name} required class="w-full rounded px-3 py-2 text-sm"
          style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />
      </div>
      <div class="flex gap-2">
        <div class="flex-1">
          <label class="block text-sm mb-1" style:color="var(--color-muted)">Host</label>
          <input bind:value={host} required class="w-full rounded px-3 py-2 text-sm"
            style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />
        </div>
        <div class="w-20">
          <label class="block text-sm mb-1" style:color="var(--color-muted)">Port</label>
          <input type="number" bind:value={port} min="1" max="65535" class="w-full rounded px-3 py-2 text-sm"
            style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />
        </div>
      </div>
      <div>
        <label class="block text-sm mb-1" style:color="var(--color-muted)">Password</label>
        <input type="password" bind:value={password} class="w-full rounded px-3 py-2 text-sm"
          style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />
      </div>
      <div>
        <label class="block text-sm mb-1" style:color="var(--color-muted)">Type</label>
        <select bind:value={connectionType} class="w-full rounded px-3 py-2 text-sm"
          style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)">
          <option value="standalone">Standalone</option>
          <option value="cluster">Cluster</option>
          <option value="sentinel">Sentinel</option>
        </select>
      </div>
      <div>
        <label class="block text-sm mb-1" style:color="var(--color-muted)">DB Index</label>
        <input type="number" bind:value={db} min="0" max="15" class="w-full rounded px-3 py-2 text-sm"
          style:background-color="var(--color-bg)" style:color="var(--color-fg)" style:border="1px solid var(--color-border)" />
      </div>
    </div>
    <div class="flex justify-end gap-2 mt-6">
      <button type="button" onclick={onclose} class="px-4 py-2 rounded text-sm"
        style:background-color="var(--color-border)" style:color="var(--color-fg)">Cancel</button>
      <button type="submit" class="px-4 py-2 rounded text-sm"
        style:background-color="var(--color-accent)" style:color="var(--color-bg)">{editing ? "Save" : "Add"}</button>
    </div>
  </form>
</div>
```

- [ ] **Step 4: Write ConnectionList**

```svelte
<!-- src/lib/components/ConnectionList.svelte -->
<script lang="ts">
  import { connections, activeConnection } from "$lib/stores/connections";
  import type { ConnectionConfig } from "$lib/types/connection";
  import ConnectionForm from "./ConnectionForm.svelte";

  let showForm = $state(false);
  let editing = $state<ConnectionConfig | null>(null);

  function handleEdit(config: ConnectionConfig) { editing = config; showForm = true; }
  function handleDelete(id: string) {
    if (confirm("Delete this connection?")) {
      connections.remove(id);
      if ($activeConnection?.id === id) activeConnection.set(null);
    }
  }
  function handleClose() { showForm = false; editing = null; }
</script>

<div class="p-4">
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-lg font-bold" style:color="var(--color-fg)">Connections</h2>
    <button onclick={() => { editing = null; showForm = true; }}
      class="rounded-md px-3 py-1 text-sm hover:opacity-80"
      style:background-color="var(--color-accent)" style:color="var(--color-bg)">+ Add</button>
  </div>

  {#if $connections.length === 0}
    <p class="text-sm" style:color="var(--color-muted)">No connections yet</p>
  {:else}
    <div class="space-y-1">
      {#each $connections as conn (conn.id)}
        <div class="flex items-center justify-between rounded px-3 py-2 cursor-pointer hover:opacity-80"
          class:bg-surface={$activeConnection?.id === conn.id}
          style:background-color={$activeConnection?.id === conn.id ? "var(--color-surface)" : "transparent"}
          onclick={() => activeConnection.set(conn)}
          role="button" tabindex="0"
          onkeydown={(e) => e.key === "Enter" && activeConnection.set(conn)}>
          <div>
            <div class="text-sm font-medium" style:color="var(--color-fg)">{conn.name}</div>
            <div class="text-xs" style:color="var(--color-muted)">{conn.host}:{conn.port}</div>
          </div>
          <div class="flex gap-1">
            <button onclick|stopPropagation={() => handleEdit(conn)} class="text-xs px-2 py-1">✏️</button>
            <button onclick|stopPropagation={() => handleDelete(conn.id)} class="text-xs px-2 py-1">🗑️</button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showForm}
  <ConnectionForm {editing} onclose={handleClose} />
{/if}
```

- [ ] **Step 5: Wire into page**

```svelte
<!-- src/routes/+page.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import ThemeToggle from "$lib/components/ThemeToggle.svelte";
  import Layout from "$lib/components/Layout.svelte";
  import ConnectionList from "$lib/components/ConnectionList.svelte";
  import { connections, activeConnection } from "$lib/stores/connections";

  onMount(() => { connections.load(); });
</script>

<Layout>
  {#snippet sidebar()}
    <div class="flex items-center justify-between px-4 pt-4"><span></span><ThemeToggle /></div>
    <ConnectionList />
  {/snippet}
  {#snippet main()}
    <div class="p-4">
      {#if $activeConnection}
        <p style:color="var(--color-fg)">Connected: {$activeConnection.name}</p>
      {:else}
        <p style:color="var(--color-muted)">Select a connection to browse keys</p>
      {/if}
    </div>
  {/snippet}
  {#snippet bottom()}
    <div class="p-4"><p class="text-sm" style:color="var(--color-muted)">Command console — Phase 5</p></div>
  {/snippet}
</Layout>
```

- [ ] **Step 6: Verify**

- Add/edit/delete connections work
- Click connection highlights as active
- Restart app → connections persist

```bash
pnpm tauri dev
```

- [ ] **Step 7: Commit**

```bash
git add . && git commit -m "feat: connection manager UI with CRUD"
```
