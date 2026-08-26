---
id: TAURI-006
title: Add Tauri Runtime Regression Matrix
status: In Progress
assignee: unassigned
parent: TAURI-000
priority: High
tags: [tauri, testing, ci]
last_updated: 2026-08-24
---

## Description

Test the daemon-client path and native windows. A Vite build or typecheck alone cannot prove IPC, window labels, file opening, media playback, or platform dialogs.

## Acceptance Criteria

- [ ] Start the packaged daemon and connect the main window
- [ ] Browse a physical directory and open a file
- [ ] Exercise grid, list, media, column, search, and recents
- [ ] Open Settings, Inspector, Quick Preview, Job Manager, and Spacedrop windows
- [ ] Verify copy, rename, folder creation, delete confirmation, and job progress
- [ ] Capture terminal state on Linux, macOS, and Windows CI runners

## Linux Runtime Validation

- The debug Tauri window connected to the existing daemon on `127.0.0.1:6969` and opened its event subscriptions.
- Overview, Recents, Favorites, File Kinds, Sources, and Redundancy rendered from the real library. Empty states were explicit where the library had no records.
- Settings opened as a second native window. The Job Manager popover and full jobs screen rendered real history with nine jobs.
- `bun run tauri:dev:no-watch` reached the native runner but GTK initialization failed on this Wayland session. Running Vite and the same debug binary with XWayland loaded the app successfully. The packaged runtime remains unproven.
- Grid, list, media, column, file opening, destructive operations, Quick Preview, Inspector, Spacedrop, and cross-platform CI remain pending because the current library has no physical location and only Linux was available.
