# Redix — Redis GUI Client

## Overview

Cross-platform desktop Redis GUI client built with Tauri v2 + TailwindCSS. Supports all Redis deployment types (standalone, cluster, sentinel) with SSH tunneling, SSL, and Teleport compatibility.

**Target platforms:** Windows, macOS, Linux

---

## 1. Connection Management

### 1.1 Multi-Connection Support
- Create, edit, delete named connections
- Quick-switch between active connections via sidebar or tab bar
- Connection groups/folders for organizing by environment (dev, staging, prod)

### 1.2 Connection Types
- **Standalone** — single Redis instance
- **Cluster** — Redis Cluster with automatic node discovery and MOVED/ASK redirect handling
- **Sentinel** — Redis Sentinel with automatic failover

### 1.3 Transport Security
- **SSH Tunnel** — connect via SSH bastion (key-based and password auth)
- **SSL/TLS** — certificate verification, CA bundle support
- **SSH + SSL** — both simultaneously

### 1.4 Credential Storage
- Encrypted local file in app config directory
- Export/import connections as encrypted JSON
- Passwords never stored in plaintext

### 1.5 Teleport Compatibility
- Auto-detect Teleport-restricted connections
- Disable restricted commands (CLUSTER, CONFIG, DEBUG, SCRIPT, etc.) automatically
- Visual indicator showing read-only/restricted mode is active
- Works without Teleport for standard Redis connections

---

## 2. Key Browser

### 2.1 Tree View (Folder-Style)
- Hierarchical display using key separator (default `:`, configurable per connection)
- Lazy-loading for large keyspaces
- Real-time key count per folder/namespace
- Search/filter keys by pattern (glob or regex)
- Refresh button to reload keyspace

### 2.2 Key Operations
- View key TTL
- Set/update TTL
- Delete single key
- Delete keys by pattern (with confirmation)
- Rename key
- Copy key name

### 2.3 Supported Redis Data Types
All types with type-aware display:

| Type | Display Format |
|------|---------------|
| **String** | Text editor with syntax highlighting (JSON, XML, etc.) |
| **Hash** | Table with key-value columns, inline editing, filter |
| **List** | Indexed table with pagination, drag-to-reorder |
| **Set** | Member list with search/filter |
| **Sorted Set** | Table with score + member columns, sort by score |
| **Stream** | Timeline view with message ID, fields, consumer groups |
| **HyperLogLog** | Count display with merge capability |
| **Bitmap** | Bit-level visualization |
| **GeoSpatial** | Map view with coordinates + distance calculations |

### 2.4 Value Display
- Auto-detect data format: JSON, XML, MessagePack, binary, plain string
- Beautify/pretty-print for structured data (JSON indented, XML formatted)
- Raw view toggle (hex for binary data)
- Copy value to clipboard
- Full-screen editor for large values

### 2.5 Value Editing
- Inline editing for simple types (string, hash fields, set members)
- Form-based editing for complex types
- Undo support for edits
- Save confirmation for destructive changes

---

## 3. Command Console

### 3.1 Manual Redis Commands
- Command input with syntax highlighting
- Auto-complete for Redis commands and arguments
- Multi-line command support (for Lua scripts, etc.)
- Execute with Enter, newline with Shift+Enter

### 3.2 Command History
- Persistent command history per connection
- Searchable history panel
- Up/Down arrow to navigate recent commands
- Pin frequently used commands

### 3.3 Output Display
- Formatted output (tables for list/set results, etc.)
- Execution time display
- Error highlighting with clear messages

---

## 4. UI/UX

### 4.1 Theme
- Dark mode (default)
- Light mode
- Toggle in header or settings
- Persist preference across sessions

### 4.2 Layout
- Resizable panels (sidebar / key browser / detail view)
- Sidebar: connection list + key tree
- Main area: value viewer/editor
- Bottom panel: command console (collapsible)

### 4.3 Loading States
- Spinner/loading indicator on every Redis command execution
- Non-blocking UI during long operations (SCAN, KEYS, etc.)
- Progress indicator for bulk operations

### 4.4 Notifications
- Toast notifications for success/error feedback
- Non-intrusive, auto-dismiss

---

## 5. Architecture & Tech Stack

| Layer | Technology |
|-------|-----------|
| Framework | Tauri v2 |
| Frontend | TailwindCSS + (Svelte, React, or Vue — TBD) |
| Redis client | `redis-rs` (Rust) or `fred` (Rust async) |
| SSH tunnel | `thrussh` or `ssh2` crate |
| State management | Tauri commands + frontend state |
| Config storage | `tauri-plugin-store` or custom encrypted file |

---

## 6. Non-Functional Requirements

### 6.1 Performance
- Lazy-load keys (SCAN-based, not KEYS)
- Virtual scrolling for large key lists (10k+ keys)
- Connection pooling for cluster mode
- Memory-efficient streaming for large values

### 6.2 Reliability
- Graceful handling of connection drops with auto-reconnect
- Timeout handling on all Redis operations
- Error boundaries in UI — one failed operation doesn't crash the app

### 6.3 Testing
- Unit tests for all Rust backend logic (connection, command parsing, data formatting)
- Integration tests for Redis operations
- Component tests for UI
- Target: 80%+ code coverage

### 6.4 Code Quality
- Clean, modular architecture
- Consistent error handling patterns
- No hardcoded values — use constants/config
- Files under 800 lines, functions under 50 lines

---

## 7. Out of Scope (v1)

- Redis ACL management UI
- Slow log / latency monitoring dashboards
- Pub/Sub interactive viewer
- Script library management
- Multi-database comparison
- Cluster rebalancing tools
