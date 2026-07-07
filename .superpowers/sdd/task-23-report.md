# Task 23: Auto-Reconnect on Connection Drop

## Status: COMPLETE

## Changes

### Rust Backend (`src-tauri/src/commands/connections.rs`)
- Added `reconnect` Tauri command that loads config from ConnectionStore by UUID, creates new StandaloneClient, connects, and replaces entry in ConnectionManager map
- Added imports for `ConnectionManager`, `StandaloneClient`, `RedisClient`
- `cargo check` passes (pre-existing deprecation warning only)

### Tauri Registration (`src-tauri/src/lib.rs`)
- Registered `commands::connections::reconnect` in invoke_handler

### Frontend (`src/lib/stores/connections.ts`)
- Added `withReconnect<T>(connId, fn)` wrapper that catches connection/refused errors, shows toast, invokes reconnect command, then retries the original operation
- Uses proper `instanceof Error` check instead of `e?.toString?.()` for type safety

## Commit
`feat: auto-reconnect on connection drop` (165b41e)
