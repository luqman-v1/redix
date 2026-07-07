# Task 24: Global Keyboard Shortcuts

## Status: DONE

## Changes

| File | Change |
|------|--------|
| `src/lib/utils/shortcuts.ts` | New file: Shortcut interface + registerShortcuts() utility |
| `src/routes/+page.svelte` | Import shortcuts + theme, register in onMount with cleanup |
| `src/lib/components/Console.svelte` | Add `data-console-input` attribute to textarea |
| `src/lib/components/KeyTree.svelte` | Add `data-key-search` attribute to filter input |

## Shortcuts Implemented

| Shortcut | Action |
|----------|--------|
| Ctrl+T | Toggle theme (dark/light) |
| Ctrl+K | Focus console input |
| Ctrl+F | Focus key search input |

## Notes

- Skipped Ctrl+N (new connection form) — requires state plumbing into ConnectionList that doesn't exist yet. Add when connection form gets a toggleable visibility prop.
- `registerShortcuts` returns cleanup function, wired into onMount return for proper teardown.
- Ctrl detection handles both Ctrl (Windows/Linux) and Cmd (macOS) via `e.ctrlKey || e.metaKey`.
