---
id: DEV-003
title: Restore Workspace Clippy on Toolchain 1.98
status: Done
assignee: bernardopg
parent: DEV-000
priority: High
milestone: M1
sprint: S01
tags: [ci, clippy, toolchain]
last_updated: 2026-10-01
---

## Description

On 2026-10-01, `cargo clippy --workspace --locked -- -D warnings` fails at HEAD with about 91 errors in crates that recent work did not touch (`archive`, `ffmpeg`, `media-metadata`, `task-system`, `crypto`), mostly `doc_markdown` and `unused_async`. Because clippy stops at the first failing dependency, `wing-core` and `wing-cli` are not linted at all under `-D warnings`. This is the CI gate.

## Acceptance Criteria

- [x] Confirm whether CI on `main` fails the same way: CI ran only `cargo check`, so clippy was never gated; it is now (`.github/workflows/ci.yml`)
- [x] Fix or explicitly allow the new lints per crate
- [x] `cargo clippy --workspace --locked -- -D warnings` passes locally

## Resolution (2026-10-01)

- Machine-applicable lints fixed with `cargo clippy --fix` (raw-pointer borrows, `const fn`, literals, `div_ceil`, `write!` newlines).
- `WingDrive` and `MacOS` added to `doc-valid-idents` (root `.clippy.toml` and `crates/media-metadata/clippy.toml`, which overrides it).
- `wing-ffmpeg` audio decoder: numeric and pointer-alignment casts allowed at module level with the reason (PCM conversion; FFmpeg aligns frame planes); resampler no longer returns a needless `Result`; channel `if` chain is a `match`.
- Hand fixes in `wing-archive`, `log-analyzer`, `wing-media-metadata`, the TypeScript generator, `wing-tauri-core`, and the Tauri app (event stream now shuts down its socket instead of dropping borrowed halves).
- Every target compiles again: `task-system` integration tests (rand 0.10 `RngExt`; `lending-stream` dropped because its `Next` future hits `todo!()` on `Pending`; `try_init` for the shared tracing subscriber) and `delete_strategy_test` (opendal 0.59 API).
- `delete_strategy_test` also exposed a real bug: the CoW detector used the disk list, which skips tmpfs, so `/tmp` was attributed to the btrfs root and secure delete was refused there. Linux now asks `statfs` for the path's own filesystem.

## Remaining

`cargo clippy --all-targets` still reports about 400 lints in test code, mostly repeated through shared helpers (`needless_borrows_for_generic_args`, `manual_checked_ops`, `type_complexity`). Not part of the gate; clean up opportunistically.
