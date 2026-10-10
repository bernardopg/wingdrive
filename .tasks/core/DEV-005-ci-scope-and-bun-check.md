---
id: DEV-005
title: "Faster CI, release and desktop builds"
status: Done
assignee: unassigned
priority: High
parent: DEV-000
tags: [development, ci, build, performance]
last_updated: 2026-10-10
---

## Description

Desktop-only pull requests paid for the whole Rust workspace in CI: the test build of
`wing-core` alone took about 7 of the 17 minutes. The 2.0.0-alpha.11 release run took 57 minutes
because one runner compiled `wing-core` four times in release mode, one after the other:

| Step | Time |
|---|---|
| `xtask setup` release daemon | 14m 36s |
| Tauri `beforeBuildCommand` rebuilding the same daemon | 8m 23s |
| Tauri app (`wing-core` without ffmpeg and heif) | 13m 40s |
| `wing-server` and `wing` (`wing-core` with `cli`) | 11m 25s |

Its Rust cache also never restored. GitHub scopes a cache saved on a tag to that tag, so each
release saved 1.8 GB that no later run could read, while it pushed other caches toward the
repository's 10 GB limit. `Swatinem/rust-cache` drops the `key` input when `shared-key` is set,
so CI and release caches differed only by their environment hash.

Bun 1.4.3 added `bun check`, a port of typescript-go. On this repository it reports the same
708 errors as `tsc` 7 in `packages/interface` (6.1 s against 1.9 s) and checks the desktop app
in 1.2 s against 2.6 s. `tsc-rs` 0.2.0 was measured at 1.4 to 1.7 s and left out: it is
experimental and ships no Linux arm64 or Windows build.

## Acceptance Criteria

- [x] `scripts/ci-scope.sh` maps changed paths to `none`, `full` or app areas (`desktop`,
  `server`, `cli`, `mobile`). CI and `just ci-local` share it.
- [x] `scripts/ci-rust.sh` runs the Rust gates for a scope. `full` runs the same commands CI ran
  before. `desktop` runs clippy on the Tauri crates and the `WingDrive` tests only.
- [x] Pushes to `main` always run the full scope.
- [x] The release workflow builds desktop bundles and server binaries on parallel runners, builds
  the release daemon once, and accepts `component: desktop | server` for verification-only runs.
  Publishing still needs both.
- [x] Release caches save only from a `workflow_dispatch` on `main`, where tag runs can read them.
- [x] Typecheck scripts use `bun check`, and every workflow pins Bun 1.4.3.
- [x] The desktop shell no longer depends on `wing-core`. It used one function,
  `default_data_dir()`, which now lives in the `wing-branding` crate with the rest of
  `core::branding`. `wing-core` re-exports it, so `wing_core::branding` paths still work. The Tauri
  app's dependency graph drops from 947 to 417 crates, which shortens `tauri dev` and the release
  bundle build.
- [x] `bun run typecheck` checks tauri, web, interface, ts-client, ai and primitives. `interface` and
  `web` failed on their own (708 and 680 errors): they lacked the Vite client types, the shared
  ambient module declarations and the Spacebot stub path. The ambient types moved to
  `packages/interface/src/env.d.ts`, which both apps reference.
- [x] The Repository Metadata job caches its Rust build instead of compiling `task-validator` cold
  (2m10s per run).
- [x] Disk cleanup on CI and release runners runs in the background during checkout and setup,
  and the build waits for it (it cost 66 to 106 s in the foreground).
- [x] Hosted runners drop every toolchain except the pinned one before the cache step, so a runner
  image update no longer changes the cache key. `core/rust-toolchain.toml`, an upstream leftover
  that asked for `stable` inside `core/`, is removed.

## Notes

The first CI run on `main` after this change starts with a cold Rust cache, because the cache key
changed. Run the release workflow once from `main` with `publish: false` to seed the release
caches before the next tag.
