---
id: TAURI-000
title: "Epic: Tauri Desktop Readiness"
status: In Progress
assignee: unassigned
parent: UI-000
priority: High
tags: [epic, tauri, desktop, quality]
last_updated: 2026-09-25
---

## Description

Make the Tauri desktop app honest and release-ready. Every visible action must execute a real backend or platform operation, report an explicit unavailable state, or stay hidden.

## Current Evidence

- Earlier TypeScript typecheck and Tauri Rust check passed. The current checkout's interface typecheck fails in third-party declarations, missing assets, and the unavailable private Spacebot client.
- Native Linux window starts and connects to the daemon.
- Overview, File Kinds, Sources, Redundancy, Settings, tabs, sidebar, inspector, and auxiliary windows render against real runtime state.
- Favorites use persisted state and are covered by TAURI-002.
- Spacebot is hidden or explicitly unavailable when its private runtime is absent; its real-runtime contract test remains open.

## Acceptance Criteria

- [x] Audit visible routes and native window entry points
- [x] Remove confirmed no-op controls from the default desktop UI
- [x] Wire the file context-menu Quick Look action to Quick Preview
- [x] Wire Sources search to the loaded source list
- [ ] Complete TAURI-002 through TAURI-007
- [ ] Run the desktop regression matrix on Linux, macOS, and Windows
- [ ] Verify a production bundle, not only the dev executable
