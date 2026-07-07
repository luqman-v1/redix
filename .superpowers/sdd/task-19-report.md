# Task 19: Command History Store

## Status: COMPLETE

## Changes

### Created
- `src-tauri/src/commands/history.rs` — Rust backend with `HistoryStore` type alias (`Arc<Mutex<HashMap<String, Vec<String>>>>`) and two commands: `get_history`, `add_to_history` (capped at 1000 entries per connection, FIFO eviction)
- `src/lib/stores/history.ts` — Svelte writable store with `load(connectionId)` and `add(connectionId, command)` methods that invoke Tauri commands

### Modified
- `src-tauri/src/commands/mod.rs` — added `pub mod history;`
- `src-tauri/src/lib.rs` — added `HistoryStore` manage() call + registered `get_history`/`add_to_history` in invoke_handler

## Verification
- `cargo check` passes (1 unrelated deprecation warning on `whoami::hostname`)
- Commit: `0c7b958 feat: command history store`
