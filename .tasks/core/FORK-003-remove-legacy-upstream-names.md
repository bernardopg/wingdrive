---
id: FORK-003
title: Replace Upstream and sd Names with WingDrive and wing
status: Done
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M1
sprint: S01
tags: [fork, branding, rename, refactor]
last_updated: 2026-10-01
---

## Description

Project rule (2026-10-01): nothing in WingDrive is named after the upstream project or carries an `sd`/`sd-` prefix. Use `wingdrive` or `wing`. This covers crates, binaries, packages, scripts, functions, types, environment variables, CLI messages, and docs.

Scope at `29c564b02`: about 20 Rust crates with `sd-*` or upstream-named packages, 8 `@sd/*` packages, about 1,275 upstream-name mentions in 350 files, about 1,500 `sd_`/`sd-` identifiers in 380 files, and the path type with about 1,000 uses.

## Result

| Before | After |
|---|---|
| `sd-core`, `sd-daemon`, `sd-server`, `sd-*` crates | `wing-core`, `wing-daemon`, `wing-server`, `wing-*` |
| `sd-cli` binary, `sd <command>` | `wing` binary (crate `wing-cli`), `wing <command>` |
| upstream-named SDK crates | `wingdrive-sdk`, `wingdrive-sdk-macros` |
| `SdPath`, `sd_path` | `WingPath`, `wing_path` (Rust, TypeScript, wire JSON) |
| `@sd/*` packages | `@wingdrive/*` |
| `SD_*` environment variables | `WING_*` |
| upstream-named client, provider, context, window globals | `WingDriveClient`, `WingDriveProvider`, `WingDriveContext`, `__WINGDRIVE__` |
| `.sdlibrary` for new libraries, upstream volume marker | `.winglibrary`, `.wingdrive-volume-id` |
| WASM host import module | `wingdrive` |

## Acceptance Criteria

- [x] 1. Rust crates and binaries renamed
- [x] 2. Rust identifiers (`WingPath*`, `wing_*`)
- [x] 3. JS packages under `@wingdrive/*`, imports, aliases, `bun.lock`
- [x] 4. Generated TypeScript types regenerated
- [x] 5. New on-disk names with legacy fallback in `core::branding` (existing `.sdlibrary` libraries and old volume markers still load)
- [x] 6. Scripts, justfile, CI workflows, mobile native module and JNI symbols
- [x] 7. Docs, comments, and user-facing strings
- [x] 8. Remaining matches are only the allowed exceptions below
- [x] 9. `cargo check --workspace`, Tauri check, `bun run typecheck`, web build, `wing-core` lib tests (365), `wing-fs-watcher` tests, branding legacy test, frontend bun tests

## Allowed Exceptions

- `LICENSE`, `NOTICE.md`, the README fork attribution, and `authors` in crate manifests (credit to the original authors).
- `whitepaper/upstream-whitepaper.*`: the upstream paper, renamed but not rewritten.
- `core/src/branding.rs` and code that reads or cleans up pre-fork data: legacy data directory, config file, keychain service, `.sdlibrary` directories, volume marker, daemon service names, voice profile key, and ignore rules for the legacy data directory.
- `scripts/check-wingdrive-independence.sh`, which lists forbidden upstream endpoints on purpose.
- Words that only contain the letters, such as `SDK`.

## Notes

- No public release exists, so environment variables and wire field names changed without a fallback. Paused jobs serialized by an older build may not resume.
- The public-share design docs pointed at the upstream share host. They now use the placeholder `wingdrive.app`; no code depends on it, and WingDrive needs its own host before shares ship.
