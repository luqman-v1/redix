# Task 2: TailwindCSS v4 + Theme Tokens

## Status: DONE

## Files Modified

| File | Action |
|------|--------|
| `package.json` / `pnpm-lock.yaml` | Added `tailwindcss@4.3.2`, `@tailwindcss/vite@4.3.2` |
| `vite.config.js` | Added `tailwindcss` import + plugin before `sveltekit()` |
| `src/app.css` | Created — `@import "tailwindcss"` + CSS custom properties (dark/light) + body styles |
| `src/routes/+layout.svelte` | Created — imports `../app.css`, wraps children with `<slot />` |

## Build Result

- `cargo build` in `src-tauri/`: PASS (11s, no errors)
- `pnpm add`: PASS, TailwindCSS v4.3.2 + vite plugin installed

## Theme Tokens (CSS Custom Properties)

Dark mode (`:root`, default): bg `#1e1e2e`, fg `#cdd6f4`, surface `#313244`, border `#45475a`, accent `#89b4fa`, muted `#6c7086`, success `#a6e3a1`, error `#f38ba8`, warning `#fab387`

Light mode (`.light` class): bg `#eff1f5`, fg `#4c4f69`, surface `#ccd0da`, border `#bcc0cc`, accent `#1e66f5`, muted `#8c8fa1`, success `#40a02b`, error `#d20f39`, warning `#df8e1d`

Both palettes are Catppuccin (Mocha dark, Latte light).

## Commit

`0fce553` — `feat: tailwindcss setup with dark/light theme tokens`

## Concerns

- No `+layout.svelte` existed, so created one with `<slot />`. Minimal; `+layout.ts` (SPA mode) unchanged.
- `cargo` not on PATH in shell hook env; confirmed build via sourcing `~/.cargo/env` first. Devs should ensure Rust toolchain in PATH.
