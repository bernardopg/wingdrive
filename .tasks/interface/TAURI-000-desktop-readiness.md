---
id: TAURI-000
title: "Epic: Tauri Desktop Readiness"
status: In Progress
assignee: unassigned
parent: UI-000
priority: High
tags: [epic, tauri, desktop, quality]
last_updated: 2026-08-24
---

## Description

Make the Tauri desktop app honest and release-ready. Every visible action must execute a real backend or platform operation, report an explicit unavailable state, or stay hidden.

## Current Evidence

- TypeScript typecheck and Tauri Rust check pass.
- Native Linux window starts and connects to the daemon.
- Overview, File Kinds, Sources, Redundancy, Settings, tabs, sidebar, inspector, and auxiliary windows render against real runtime state.
- Favorites and the standalone Search route are placeholders.
- Spacebot uses a no-op compatibility client when its private runtime is absent.

## Acceptance Criteria

- [x] Audit visible routes and native window entry points
- [x] Remove confirmed no-op controls from the default desktop UI
- [x] Wire the file context-menu Quick Look action to Quick Preview
- [x] Wire Sources search to the loaded source list
- [ ] Complete TAURI-002 through TAURI-007
- [ ] Run the desktop regression matrix on Linux, macOS, and Windows
- [ ] Verify a production bundle, not only the dev executable
