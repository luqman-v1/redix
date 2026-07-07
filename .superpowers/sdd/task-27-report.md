# Task 27: Virtual Scrolling for Large Key Lists

**Status:** Complete
**Commit:** `4fad3a2` — `perf: virtual scrolling for large key lists`

## Changes

### KeyTree.svelte (`src/lib/components/KeyTree.svelte`)
- All keys loaded via SCAN stored in `allKeys[]` (unchanged SCAN behavior)
- `displayedCount` tracks how many keys are shown (starts at 500)
- `buildTree()` called only on the visible slice (`allKeys.slice(0, displayedCount)`)
- "Load more..." button appears when more keys exist, loads next 500
- Status text: "Showing X of Y keys" replaces old "{count} keys"
- Button styled to match existing UI (border, accent color, hover state)

### Console.svelte (`src/lib/components/Console.svelte`)
- `MAX_ENTRIES = 100` constant added
- Both success and error paths slice to last 100 entries: `entries = [...entries, entry].slice(-MAX_ENTRIES)`
- Prevents unbounded DOM growth during long sessions

## What was skipped
- Full virtual scroll (IntersectionObserver/viewport-based) — not needed at v1, pagination simpler and sufficient
- No new dependencies added (no `svelte-virtual-list` or `@tanstack/svelte-virtual`)
