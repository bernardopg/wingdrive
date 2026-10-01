---
id: FORK-003
title: Replace Spacedrive and sd Names with WingDrive and wing
status: To Do
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M1
sprint: S01
tags: [fork, branding, rename, refactor]
last_updated: 2026-10-01
---

## Description

Project rule (2026-10-01): nothing in WingDrive is named "Spacedrive" or carries an `sd`/`sd-` prefix. Use `wingdrive` or `wing`. This covers crates, binaries, packages, scripts, functions, types, environment variables, CLI messages, and docs.

Scope measured at `29c564b02`: about 20 Rust crates named `sd-*` or `spacedrive-*`, 8 `@sd/*` packages, about 1,275 "Spacedrive" mentions in 350 files, about 1,500 `sd_`/`sd-` identifiers in 380 files, and `SdPath` with about 1,000 uses.

## Rules

- Keep legally required upstream attribution only (license headers, NOTICE/README credit that WingDrive derives from Spacedrive).
- On-disk names (`.sdlibrary`, data directories, `SD_*` environment variables) get the new name, and the old one is still read with a one-time migration or a fallback, so existing libraries keep working.
- Wire method strings and serialized enum tags that do not contain `sd` stay unchanged.

## Phases

- [ ] 1. Rust crates and binaries: `wing-core` -> `wing-core`, `wing-cli` -> `wing-cli` (binary `wing`), `wing-daemon` -> `wing-daemon`, `wing-server` -> `wing-server`, `sd-*` libs -> `wing-*`, `wingdrive-sdk*` -> `wingdrive-sdk*`
- [ ] 2. Rust identifiers: `SdPath*` -> `WingPath*`, `sd_*` functions/modules -> `wing_*`
- [ ] 3. JS packages: `@sd/*` -> `@wingdrive/*` (or `@wing/*`), imports, Vite aliases, tsconfig paths
- [ ] 4. Generated TS/Swift types regenerated
- [ ] 5. Env vars `SD_*` -> `WING_*` with fallback; `.sdlibrary` -> `.winglibrary` with migration
- [ ] 6. Scripts, justfile, xtask, CI workflows, mobile modules (`wing-mobile-core`)
- [ ] 7. Docs, comments, and user-facing strings
- [ ] 8. `git grep -iE 'spacedrive|\bsd[-_]|\bSd[A-Z]'` returns only allowed attribution and migration code
- [ ] 9. `cargo check --workspace`, `bun run typecheck`, tests, and a runtime smoke pass
