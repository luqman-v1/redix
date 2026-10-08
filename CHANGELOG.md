# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Nothing yet.

## [0.3.0] - 2026-09-26

Performance work on the Redis path, a UI bug that made the app unusable
without a stored connection, and a first pass at update visibility.

### Added

- **Update banner.** The sidebar now reports when a newer GitHub release
  exists and links to it. Drafts and prereleases are ignored, and the probe
  is cached for six hours so the unauthenticated GitHub API rate limit is
  not exhausted by repeated launches. Downloads stay manual: the release
  pipeline is unsigned, so a real auto-updater would not be installable.
- **Test coverage.** 39 Rust unit tests, 23 frontend unit tests, 10 Playwright
  specs and 6 Redis integration tests, covering the helpers introduced by
  the performance work below.
- **Connection database switch.** Switching DB on an open connection no longer
  discards the open tabs.

### Fixed

- **The connection modal could not be dismissed.** The auto-open effect read
  `showConnections` alongside `active`, so closing the dialog immediately
  reopened it. With no stored connection the backdrop stayed up permanently
  and the theme toggle, connection selector and "Connect Now" action were
  all unclickable.

### Changed

- **Key scan is pipelined.** `scan_keys` issued one `TTL` round-trip per key;
  a 1000-key page meant 1000 requests. It now sends a single pipeline, and a
  new `get_key_meta` command returns type and TTL together so opening a key
  costs one round-trip instead of two. Verified against a real Redis to
  confirm the pipelined values match the per-key ones exactly.
- **Key scan is bounded.** A wildcard pattern could pull 200k keys into
  memory and rebuild the whole tree on every page. It is now capped at 10k
  keys, with a hint to refine the pattern.
- **TTL countdown uses a single interval.** Every visible key subscribed to a
  shared ticker, so a large tree meant hundreds of callbacks per second. One
  timer in the key tree now drives them all.
- **Credential handling.** Four near-identical URL builders, none of which
  escaped credentials, were collapsed into one. Passwords containing `@`,
  `:` or `/` previously produced a malformed connection URL.
- **Read-only enforcement is centralised.** Twelve-plus inline copies with
  three different error messages now share one guard, which also applies to
  pipelines. A mixed read/write pipeline is rejected outright rather than
  partially applied.
- **Config store is cached.** PBKDF2 derivation ran on every store
  construction and the encrypted file was re-read and decrypted on every
  mutation. Both are now done once and cached, refreshed on write.
- **Command log noise.** `PING`, `SCAN`, `TYPE`, `TTL`, `INFO` and pipelines
  were emitting a Tauri event per call, spamming IPC on every dashboard poll
  and tree render. They are filtered out.
- **Memory analyzer bounds.** `sample_size` is clamped to 50k and processed
  in smaller chunks, so an oversized request no longer monopolises the
  connection. Pipeline failures are logged instead of printed.
- **Dashboard polling pauses when the window is hidden**, removing twelve
  `INFO` round-trips per minute per idle connection.
- **Build and release.** Migrated from pnpm to bun, which also fixed
  `bun run test:e2e`, previously unable to start without pnpm installed.
  Release bundling is scoped per platform instead of building every format
  on every runner. Dependency trimming removed `ssh2` (and its `libssh2`
  C dependency), `once_cell` and unused `rustls` crates.
- **Screenshots are WebP**, cutting 272 KB to 67 KB in the repository.

### Removed

- Dead SSH tunnel implementation. `SshTunnel` bound a local port, dropped
  the listener and the channel, and had no caller. The `ssh` field on a saved
  connection is still accepted for compatibility but is ignored.
- Scratch files left in `src-tauri/` by earlier experiments.

### Known issues

- macOS builds are ad-hoc signed, so Gatekeeper reports the app as damaged.
  A Developer ID signature and notarization are required before distributing
  a `.dmg` publicly.

## [0.2.1] - 2026-07-27

## [0.2.0]

## [0.1.0]