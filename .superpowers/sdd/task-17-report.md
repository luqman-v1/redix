# Task 17: Type-Aware Value Viewers

## Status: COMPLETE

## Files Created

| File | Purpose |
|------|---------|
| `src/lib/components/viewers/StringViewer.svelte` | String value with format badge (json/xml/binary/text), pretty-print, edit/save |
| `src/lib/components/viewers/HashViewer.svelte` | Hash fields table with search filter |
| `src/lib/components/viewers/ListViewer.svelte` | Indexed list table with push head/tail |
| `src/lib/components/viewers/SetViewer.svelte` | Set members with filter, add, delete |
| `src/lib/components/viewers/SortedSetViewer.svelte` | Score/Member table with column sorting |
| `src/lib/components/viewers/StreamViewer.svelte` | Expandable stream entries with field tables |
| `src/lib/components/viewers/GeoViewer.svelte` | Member/Longitude/Latitude/Score table |
| `src/lib/components/viewers/HyperLogLogViewer.svelte` | Cardinality count display |
| `src/lib/components/viewers/BitmapViewer.svelte` | Hex dump placeholder |
| `src/lib/components/viewers/ValueViewer.svelte` | Router: type switch -> correct viewer |

## Design Decisions

- **Svelte 5 runes** throughout (`$state`, `$props`, `$derived.by`, `$effect`)
- **CSS custom properties** for all colors (`--color-fg`, `--color-border`, `--color-surface`, `--color-accent`, `--color-error`, `--color-muted`, `--color-input-bg`, `--color-surface-raised`)
- **Tauri invoke param naming** matches Rust snake_case (`connection_id`/`connectionId` handled by Tauri serde)
- **SortedSetViewer** uses `$derived.by` for sorted copy (immutable pattern)
- **StreamViewer** uses expandable entries (lazy field display)
- **BitmapViewer** reuses `toHex()` from beautifier - full bitmap viz deferred
- All viewers follow same pattern: loading state -> error display -> content

## Verification

- `svelte-check`: 0 errors, 0 warnings
- Type mapping in ValueViewer: string, hash, list, set, zset, stream, geo, hyperloglog, bitmap
