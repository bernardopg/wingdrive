---
id: DEV-002
title: "Isolate Linux desktop development runtime"
status: In Progress
assignee: unassigned
priority: High
parent: DEV-000
tags: [development, tauri, daemon, safety, linux]
last_updated: 2026-09-03
---

## Description

Keep local desktop development separate from production WingDrive data and its
standard daemon. A development launch must choose an explicit data root and
named daemon instance before it can connect or create state.

## Acceptance Criteria

- [x] `scripts/dev-isolated.sh` starts Tauri with a dedicated data root and named instance.
- [x] Tauri derives its effective data directory and daemon address from `WINGDRIVE_DATA_DIR` and `WINGDRIVE_INSTANCE`.
- [x] The development daemon receives both `--data-dir` and `--instance`.
- [x] The development bootstrap rejects absent or invalid isolation variables before the daemon starts.
- [x] Documentation explains custom roots, direct CLI use, and safe cleanup.
- [x] Start default-style and named-instance daemons concurrently on Linux against disposable roots; confirm ports 6969 and 7096 and separate directories, with the corrected binary.
- [x] Run Tauri against the isolated daemon while a production-style daemon runs, then confirm libraries and device identities remain independent.
- [x] Add unit tests for runtime configuration and named-instance port selection.
- [x] Add an integration test that starts the Tauri-selected daemon and verifies its arguments.

## Validation

2026-09-03 (first pass, stale binary): `target/debug/sd-daemon` ran a
disposable default-style daemon on port 6969 and `desktop-dev` on port 7096;
directories stayed separate. The binary in use was later found stale.

2026-09-03 (corrected binary, rebuilt): same concurrent check repeated.
Default-style daemon wrote directly under its root; the `desktop-dev` instance
wrote only under `instances/desktop-dev/`; the dev root contained nothing but
`instances/`. Both instances wrote `wingdrive.json`. Ports 6969 and 7096 were
reachable simultaneously.

2026-09-03 (tests):

- `cargo test -p sd-core --lib config::app_config` — 2 passed, including the
  new `fresh_directory_creates_wingdrive_config_not_legacy` regression test.
- `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml --bin WingDrive`
  — 3 passed, including the new integration test that spawns the real
  `sd-daemon` through the Tauri command builder and asserts reachability on the
  derived port plus structural isolation under `instances/<name>/`.
- Task validator, Tauri typecheck, `cargo fmt --check`, JSON parsing of docs
  navigation, and `git diff --check` all passed.

Environment lesson recorded for DEV-000: running two cargo invocations
concurrently against one target directory produced a stale-looking `sd-daemon`
artefact whose behavior did not match current source (it still wrote the
legacy config name on fresh directories). Rebuilding with a single cargo
process restored correct behavior. Always rebuild before validating, and never
run overlapping cargo commands on the same target directory.

Remaining: none for the isolation itself. Track the stale-artefact hazard under
DEV-000 (see Environment lesson below).

2026-09-03 (end-to-end, Linux desktop, real GUI):

A production-style daemon ran on port 6969 with a disposable root while
`./scripts/dev-isolated.sh` launched the full Tauri session (`WINGDRIVE_DEV_ROOT`
to a disposable directory, `GDK_BACKEND=x11` for the known Wayland GTK issue).
Results:

- Dev daemon reachable on 127.0.0.1:7096; production stand-in healthy on 6969
  for the whole session; vite on 1420; `WingDrive` process alive and connected.
- The app drove the dev daemon end to end: bundled adapters were installed into
  `.../instances/desktop-dev/libraries/My Library.sdlibrary`, and
  `sd-cli --data-dir <dev-root>/data --instance desktop-dev library list`
  returned the dev library UUID.
- Device identities: real `~/.wingdrive`, real `~/.spacedrive`, the production
  stand-in, and the dev instance produced four distinct `device_id` UUIDs.
- Real data untouched: md5 snapshots of all 41 files in `~/.wingdrive` and all
  48 files in `~/.spacedrive` were byte-identical before and after the session.
- Teardown via process-group SIGTERM/SIGKILL freed ports 6969, 7096, and 1420
  and left no leftover processes.

Second Environment lesson, recorded for DEV-000: a stale 44 MB `sd-daemon`
artefact with pre-rename behavior (writes `spacedrive.json` on fresh
directories) reappears in `target/debug` after builds in the Tauri app context
(observed after `cargo test --manifest-path apps/tauri/src-tauri/Cargo.toml`
and again during a `tauri dev` session). Mitigation: `rm target/debug/sd-daemon && cargo build --bin sd-daemon` (single cargo process) restores the correct
cached artefact; smoke-test with `--data-dir <tmp> --instance t` and confirm
`wingdrive.json` before trusting the binary. The session's own daemon and the
rebuilt artefact both behaved correctly; the production stand-in launched from
the stale artefact showed the legacy filename, which is cosmetic on a fresh
directory but proves why release validation must rebuild first.
