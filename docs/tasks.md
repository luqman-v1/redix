# Redix — Task Breakdown

Derived from [PRD](./prd.md). Tasks ordered by dependency; each task is independently shippable.

---

## Phase 0: Project Scaffold

### T0.1 — Tauri v2 + Frontend Bootstrap
- Init Tauri v2 project with chosen frontend framework
- TailwindCSS setup with dark/light theme tokens
- Project structure: `src-tauri/` (Rust) + `src/` (frontend)
- Dev server + hot reload working
- **Deliverable:** blank app launches on macOS, Windows, Linux

### T0.2 — Theme System
- Design token system (colors, spacing, typography) for dark + light mode
- Theme toggle component with persistence (localStorage)
- CSS variables or Tailwind dark mode classes
- **Deliverable:** toggle switches theme instantly, persists across restarts

### T0.3 — Layout Shell
- Three-panel layout: sidebar (connection list), main area (key browser + detail), bottom (console)
- Resizable panel dividers
- Responsive minimum sizes
- **Deliverable:** app shows three empty panels with draggable dividers

---

## Phase 1: Connection Management

### T1.1 — Connection Config Model (Rust)
- `ConnectionConfig` struct: name, host, port, db, auth, ssh, ssl, type (standalone/cluster/sentinel), separator
- Serialization/deserialization (serde)
- Validation logic
- Unit tests
- **Deliverable:** config model with tests passing

### T1.2 — Encrypted Credential Storage (Rust)
- Encrypt/decrypt connection configs to local file (AES-256 or similar)
- App config directory detection (platform-specific)
- Export/import as encrypted JSON
- Unit tests for encrypt/decrypt round-trip
- **Deliverable:** connections save/load encrypted

### T1.3 — Connection Manager UI
- Sidebar: list saved connections with status indicator (connected/disconnected)
- Add/Edit connection form (modal or panel)
- Fields: name, host, port, password, db index, connection type, separator
- SSH config: host, port, user, key file / password
- SSL config: enable toggle, CA cert path, client cert/key
- Delete connection with confirmation
- **Deliverable:** full CRUD for connections in sidebar

### T1.4 — Connection Switching
- Click connection to connect/disconnect
- Active connection highlight in sidebar
- Connection status indicator (green/red/yellow)
- Multiple connections can be open simultaneously
- Tab bar or breadcrumb showing active connection + db
- **Deliverable:** switch between connections, visual feedback

---

## Phase 2: Redis Backend

### T2.1 — Redis Client Core (Rust)
- Abstract `RedisClient` trait: connect, disconnect, execute command, ping
- Standalone implementation using `redis-rs` or `fred`
- Connection pooling
- Command timeout handling
- Unit tests with mock or test container
- **Deliverable:** connect to standalone Redis, run arbitrary commands

### T2.2 — SSH Tunnel Support (Rust)
- SSH tunnel establishment before Redis connect
- Key-based auth (RSA, ED25519) + password auth
- Tunnel lifecycle (create on connect, destroy on disconnect)
- Unit tests for config parsing
- **Deliverable:** connect to Redis through SSH bastion

### T2.3 — SSL/TLS Support (Rust)
- TLS connection with certificate verification
- CA bundle, client cert, client key config
- Self-signed cert option (skip verify toggle)
- **Deliverable:** connect to Redis over TLS

### T2.4 — Cluster Mode (Rust)
- Redis Cluster client with MOVED/ASK redirect handling
- Automatic node discovery (CLUSTER SLOTS / CLUSTER NODES)
- Slot-aware command routing
- **Deliverable:** connect to Redis Cluster, commands route correctly

### T2.5 — Sentinel Mode (Rust)
- Sentinel discovery of master/replica
- Automatic failover handling
- Read from replica option
- **Deliverable:** connect via Sentinel, survives failover

### T2.6 — Teleport Detection & Command Restriction
- Auto-detect Teleport by attempting restricted command (e.g., `COMMAND INFO` or `CLUSTER INFO`)
- Maintain list of restricted commands per connection
- Tauri command to query restriction status
- Block restricted commands in console with clear error message
- **Deliverable:** Teleport connections auto-flag restricted commands

---

## Phase 3: Key Browser

### T3.1 — Key Scanning (Rust + Frontend)
- SCAN-based key loading (not KEYS) with cursor pagination
- Configurable COUNT per scan
- Key separator-based namespace tree building (e.g., `user:123:name` → folder `user` → `123` → leaf `name`)
- Tauri command: `get_keys(cursor, count, pattern, separator)`
- **Deliverable:** tree view populates with folder hierarchy

### T3.2 — Tree View Component
- Collapsible folder nodes
- Leaf nodes show key name + type icon
- Lazy-load children on expand
- Virtual scrolling for large lists (10k+)
- Key count badge per folder
- Right-click context menu (delete, rename, copy, TTL)
- **Deliverable:** navigate keyspace like a file explorer

### T3.3 — Key Search & Filter
- Search bar above tree view
- Glob pattern filter (e.g., `user:*`)
- Regex mode toggle
- Debounced search (avoid flooding Redis)
- Result count display
- **Deliverable:** filter keys by pattern in real-time

### T3.4 — Key Operations (Rust + UI)
- `DEL` single key (with confirmation)
- `DEL` by pattern (bulk, with confirmation dialog showing count)
- `RENAME` key (inline rename input)
- `TTL` display + set/update TTL (seconds, milliseconds, persist)
- Copy key name to clipboard
- **Deliverable:** right-click key → delete/rename/TTL/copy

---

## Phase 4: Value Display & Edit (per type)

### T4.1 — String Viewer/Editor
- Auto-detect format: JSON, XML, plain text, binary
- JSON: pretty-print with syntax highlighting + collapsible tree
- XML: formatted with highlighting
- Binary: hex view
- Plain text: monospace editor
- Edit + save, with confirmation on large values
- **Deliverable:** click string key → formatted value shown, editable

### T4.2 — Hash Viewer/Editor
- Table with Field | Value columns
- Inline editing of fields/values
- Add new field, delete field
- Filter/search fields
- Sort by field name
- **Deliverable:** hash displayed as editable table

### T4.3 — List Viewer/Editor
- Indexed table (index | value)
- Pagination for large lists
- Edit value at index
- Push to head/tail, pop from head/tail
- Delete by index
- **Deliverable:** list displayed as paginated table

### T4.4 — Set Viewer/Editor
- Member list with search/filter
- Add member, delete member
- Copy member value
- **Deliverable:** set displayed as searchable member list

### T4.5 — Sorted Set Viewer/Editor
- Table with Score | Member columns
- Sort by score or member
- Add member with score, update score
- Delete member
- Range queries (by score, by rank)
- **Deliverable:** sorted set as sortable table

### T4.6 — Stream Viewer
- Timeline/list view with Message ID + fields
- Paginated by message range
- Consumer group info display
- **Deliverable:** stream messages shown in timeline

### T4.7 — HyperLogLog Viewer
- Display PFCOUNT result
- Merge multiple HLLs
- **Deliverable:** HLL shows count, merge action available

### T4.8 — Bitmap Viewer
- Visual bit grid or hex dump
- Show BITCOUNT
- Get/set individual bits
- **Deliverable:** bitmap shown as bit visualization

### T4.9 — GeoSpatial Viewer
- Table with member, longitude, latitude, score
- Distance calculation between members
- Radius search
- **Deliverable:** geo set shown as coordinate table

### T4.10 — Value Beautifier (shared)
- Format detection utility (JSON, XML, MessagePack, binary)
- Pretty-print formatter
- Raw/hex toggle
- Copy to clipboard
- Full-screen editor modal for large values
- **Deliverable:** shared beautify/format component used by all type viewers

---

## Phase 5: Command Console

### T5.1 — Command Input Component
- Command input with Redis syntax highlighting
- Multi-line mode (Shift+Enter for newline)
- Execute on Enter
- Auto-complete dropdown for Redis commands
- **Deliverable:** type and execute Redis commands

### T5.2 — Command History (Rust + Frontend)
- Persist history per connection (local storage or file)
- History panel (toggle open/close)
- Search through history
- Up/Down arrow in input to cycle recent commands
- Pin favorite commands
- **Deliverable:** command history persists, searchable, navigable

### T5.3 — Command Output Display
- Formatted output per type (table for arrays, inline for scalars)
- Execution time in ms
- Error display with Redis error message
- Copy output
- Clear console
- **Deliverable:** command results formatted and timed

---

## Phase 6: Polish & Cross-Cutting

### T6.1 — Loading & Progress States
- Spinner on every async Redis operation
- Non-blocking UI during SCAN / large operations
- Cancel button for long-running operations
- Progress bar for bulk delete / bulk operations
- **Deliverable:** no operation runs without visual feedback

### T6.2 — Toast Notifications
- Success toast (key deleted, connection saved, etc.)
- Error toast (connection failed, command error, etc.)
- Warning toast (Teleport restricted command blocked, etc.)
- Auto-dismiss after 3-5s, manual dismiss
- **Deliverable:** non-intrusive notification system

### T6.3 — Auto-Reconnect
- Detect connection drops
- Exponential backoff reconnect attempt
- Visual indicator (reconnecting spinner)
- Queue commands during reconnect (optional)
- **Deliverable:** lost connections recover automatically

### T6.4 — Keyboard Shortcuts
- Cmd/Ctrl+N: new connection
- Cmd/Ctrl+K: command console focus
- Cmd/Ctrl+F: key search focus
- Cmd/Ctrl+T: theme toggle
- Cmd/Ctrl+W: close current connection tab
- Standard shortcuts (copy, paste, undo, select all)
- **Deliverable:** keyboard-driven workflow

---

## Phase 7: Testing & Optimization

### T7.1 — Rust Backend Unit Tests
- Connection config model tests
- Encryption round-trip tests
- Redis command parsing tests
- Data format detection tests
- Tree builder tests (separator-based namespace)
- **Coverage target:** 80%+

### T7.2 — Redis Integration Tests
- Test container setup (Docker Redis)
- Standalone, cluster, sentinel test suites
- SSH tunnel test
- SSL connection test
- Teleport command restriction test
- **Coverage target:** critical paths 100%

### T7.3 — Frontend Component Tests
- Theme toggle
- Connection form validation
- Key tree rendering + expand/collapse
- Value viewer for each type
- Command console input/output
- **Coverage target:** 80%+

### T7.4 — Performance Optimization
- Virtual scrolling for 10k+ keys
- Lazy value loading (don't fetch until key selected)
- Connection pooling in cluster mode
- Memory profiling for large keyspaces
- Bundle size audit
- **Deliverable:** smooth at 50k keys, < 200MB memory

### T7.5 — E2E Smoke Tests
- Launch app → add connection → connect → browse keys → view value → edit → disconnect
- Dark/light mode toggle persists
- Command console execute + history
- **Deliverable:** critical path automated

---

## Dependency Graph

```
T0.1 → T0.2 → T0.3 (scaffold sequential)
T0.1 → T1.1 → T1.2 → T1.3 → T1.4 (connection depends on scaffold)
T1.1 → T2.1 → T2.2, T2.3, T2.4, T2.5 (redis backend depends on config model)
T2.1 → T2.6 (teleport depends on redis core)
T2.1 → T3.1 → T3.2, T3.3, T3.4 (key browser depends on redis client)
T2.1 → T4.1 .. T4.9 (type viewers depend on redis client)
T4.10 → T4.1 .. T4.9 (beautifier shared by all viewers)
T2.1 → T5.1 → T5.2, T5.3 (console depends on redis client)
T6.x parallel (polish independent of each other)
T7.x after feature complete (testing phase)
```

**Parallelizable:**
- T2.2 + T2.3 + T2.4 + T2.5 (transport modes independent)
- T4.1 .. T4.9 (type viewers independent, share T4.10)
- T6.1 .. T6.4 (polish items independent)
- T7.1 + T7.2 + T7.3 (test suites independent)
