---
id: TAURI-000
title: "Epic: Tauri Desktop Readiness"
status: In Progress
assignee: bernardopg
parent: UI-000
priority: High
milestone: M3
tags: [epic, tauri, desktop, quality]
last_updated: 2026-11-15
sprint: S06
---

## Description

Make the Tauri desktop app honest and release-ready. Every visible action must execute a real backend or platform operation, report an explicit unavailable state, or stay hidden.

## Current Evidence

- Earlier TypeScript typecheck and Tauri Rust check passed. The current checkout's interface typecheck fails in third-party declarations, missing assets, and the unavailable private Spacebot client.
- Native Linux window starts and connects to the daemon.
- Overview, File Kinds, Sources, Redundancy, Settings, tabs, sidebar, inspector, and auxiliary windows render against real runtime state.
- Favorites use persisted state and are covered by TAURI-002.
- Spacebot is hidden or explicitly unavailable when its private runtime is absent; its real-runtime contract test remains open.
- Windows (TAURI-016, Done 2026-11-05): build, tests and installers in CI; `v3.0.0` published the `.exe` and `.msi`.

## Remaining children

Order decided on 2026-11-15, all in S06 unless noted:

1. REL-001: the Tauri updater key and signed `latest.json` first, so installed apps receive fixes
   without a manual download; then Authenticode and Apple.
2. TAURI-018: the Explorer smoke on every pull request, so an Explorer regression fails before a tag.
3. TAURI-012: the regression matrix by hand, Windows first (with CORE-014). macOS needs a Mac; without
   one it moves to S07.
4. TAURI-017: administrator retries on Windows (after REL-001's certificate).

## Acceptance Criteria

- [x] Audit visible routes and native window entry points
- [x] Remove confirmed no-op controls from the default desktop UI
- [x] Wire the file context-menu Quick Look action to Quick Preview
- [x] Wire Sources search to the loaded source list
- [x] Complete TAURI-002 through TAURI-007, except TAURI-004 (Spacebot), which moved to M5
- [x] Run the desktop regression matrix on Linux (TAURI-006)
- [ ] Run it on macOS and Windows (TAURI-012, M3)
- [ ] Explorer smoke runs on every pull request (TAURI-018)
- [ ] Installed apps update themselves from signed releases (REL-001)
- [x] Verify a production bundle, not only the dev executable (TAURI-011, TAURI-015)
