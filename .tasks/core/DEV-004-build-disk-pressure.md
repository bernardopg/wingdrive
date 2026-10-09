---
id: DEV-004
title: "Cut SSD writes and heat from local Rust builds"
status: Done
assignee: unassigned
priority: High
parent: DEV-000
tags: [development, build, performance, linux]
last_updated: 2026-10-09
---

## Description

On 2026-10-08 the NVMe that holds `target/` hit its critical temperature twice during wingdrive
builds, and the machine hung once. `target/` had reached 266 GB: 162 GB of incremental caches,
77 of them for `wing_core` at about 2.4 GB each, one per feature set. Every recipe enabled a
different feature set, full-width builds pushed the 32 GB machine into swap on the same SSD, and
nothing removed stale caches.

## Acceptance Criteria

- [x] `just dev-server`, `just cli` and `just build` use `--features ffmpeg,heif` like `just dev-daemon`
  and Tauri dev, so all of them reuse one `wing-core` build (verified: `wing-server` and `wing`
  build in 3 s and 7 s after `wing-daemon`, with no `wing-core` recompile).
- [x] `cargo xtask setup` writes `[build] jobs` as half the CPUs outside CI and every CPU on CI;
  `CARGO_BUILD_JOBS` still overrides it. A unit test renders the template.
- [x] `just clean-stale` removes incremental caches and artifacts untouched for 7 days
  (first run: `target/` went from 266 GB to 112 GB).

## Notes

Turning off incremental compilation for `wing-core` was measured and rejected: a rebuild after
touching `core/src/lib.rs` took 180 s and wrote about 10.5 GB, against 22 s and 1.7 GB with
incremental on, because every one of the 256 codegen units is rewritten.
