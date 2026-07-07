# Task 7: Tauri Commands for Connection CRUD

## Status: DONE

## What was created

1. **`src-tauri/src/commands/mod.rs`** — Module declaration for `connections`
2. **`src-tauri/src/commands/connections.rs`** — 5 Tauri commands:
   - `get_connections` — loads all connections from store
   - `add_connection` — adds config, returns it back
   - `update_connection` — updates config by id, returns it back
   - `delete_connection` — parses string id to UUID, deletes from store
   - `test_connection` — validates host non-empty and port > 0 (stub, no actual TCP)
3. **`src-tauri/src/lib.rs`** — Updated:
   - Added `pub mod commands`
   - Added `.manage(ConnectionStore::default())` to Builder
   - Removed placeholder `greet` command
   - Registered all 5 connection commands in `invoke_handler`

## Decisions

- `add_connection`/`update_connection` return `ConnectionConfig` back to frontend for confirmation
- `delete_connection` accepts `String` id from frontend, parses to `Uuid` internally
- `test_connection` is a validation stub — actual TCP/Redis ping deferred to connection pool task
- Pre-existing `whoami::hostname()` deprecation warning in store.rs left untouched (not this task's scope)

## Build

`cargo build` passes with 1 pre-existing deprecation warning (unrelated).

## Commit

`05fed70` — `feat: tauri commands for connection CRUD`
