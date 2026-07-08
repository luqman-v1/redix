# UI Bug Fixes Report

## Changes Summary

Commit: `dd969c9` - `fix: UI bugs - wire ValueViewer, fix layout, fix leaks`

## Files Modified (7)

1. **`src/routes/+page.svelte`** - Wire ValueViewer to key selection, fix store subscription leak
2. **`src/lib/components/ConnectionList.svelte`** - Fix store subscription leak
3. **`src/lib/components/Console.svelte`** - Fix Shift+Enter, fix store subscription leak
4. **`src/lib/components/Panel.svelte`** - Fix resize handle hit area
5. **`src/lib/utils/tree-builder.ts`** - Filter empty segments from key split
6. **`src/app.css`** - Add box-sizing reset, global styles

## Fixes Applied

### 1. ValueViewer wired to key selection
- `handleKeySelect` now calls `invoke("get_key_type", ...)` and stores result
- Renders `<ValueViewer>` with key name header + type badge when key selected
- Falls back to connection info when no key selected

### 2. Store subscription memory leaks fixed
- `+page.svelte`: `activeConnection.subscribe(...)` replaced with `$derived(get(activeConnection))`
- `ConnectionList.svelte`: Both `connections.subscribe(...)` and `activeConnection.subscribe(...)` replaced with `$derived(get(...))`
- `Console.svelte`: `history.subscribe(...)` replaced with `$derived(get(history))`
- All use `get` from `svelte/store` for one-shot reads in `$derived`

### 3. Panel resize handle
- Separator div expanded from 1px to 6px hit area (visual stays 1px via opacity)
- Added hover effect: opacity goes to 1 on hover (accent color highlight)

### 4. Shift+Enter in Console
- Removed `preventDefault` block that suppressed Shift+Enter
- Enter without Shift now correctly calls `execute(input)` with `await`

### 5. Tree builder empty segments
- Added `.filter(p => p.length > 0)` after `key.split(separator)`
- Prevents empty string nodes from keys like `:foo:bar` or trailing separators

### 6. Global CSS
- Added `box-sizing: border-box` reset for all elements
- Added `html, body { height: 100%; overflow: hidden }` for full-viewport layout
- Added thin scrollbar styling (both Firefox and WebKit)
- Added default button/input resets

## Verification

- Build passes (10 files changed, 239 insertions, 47 deletions)
- All Svelte 5 syntax: `$state()`, `$derived()`, `$props()`, `$effect()`
