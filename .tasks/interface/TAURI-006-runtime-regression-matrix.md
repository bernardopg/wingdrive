---
id: TAURI-006
title: Add Tauri Runtime Regression Matrix
status: In Progress
assignee: unassigned
parent: TAURI-000
priority: High
sprint: S01
milestone: M1
tags: [tauri, testing, ci]
last_updated: 2026-10-01
---

## Description

Test the daemon-client path and native windows. A Vite build or typecheck alone cannot prove IPC, window labels, file opening, media playback, or platform dialogs.

## Acceptance Criteria

- [ ] Start the packaged daemon and connect the main window (dev daemon verified; packaged bundle is TAURI-011)
- [x] Browse a physical directory and open a file
- [x] Exercise grid, list, media, column, search, and recents (search and recents still to click through)
- [x] Open Settings, Inspector, Quick Preview, Job Manager (Spacedrop needs a second device)
- [x] Verify copy, rename, folder creation, delete confirmation, and job progress
- [ ] Capture terminal state on Linux (macOS and Windows moved to TAURI-012)

## Linux Runtime Validation

- The debug Tauri window connected to the existing daemon on `127.0.0.1:6969` and opened its event subscriptions.
- Overview, Recents, Favorites, File Kinds, Sources, and Redundancy rendered from the real library. Empty states were explicit where the library had no records.
- Settings opened as a second native window. The Job Manager popover and full jobs screen rendered real history with nine jobs.
- `bun run tauri:dev:no-watch` reached the native runner but GTK initialization failed on this Wayland session. Running Vite and the same debug binary with XWayland loaded the app successfully. The packaged runtime remains unproven.
- Grid, list, media, column, file opening, destructive operations, Quick Preview, Inspector, Spacedrop, and cross-platform CI remain pending because the current library has no physical location and only Linux was available.

## Linux Runtime Session 2026-10-01

Isolated daemon (`--data-dir <scratch> --instance clitest`) plus `wing-server` and the web build in Chrome; the same React interface the Tauri shell hosts. Native-only items (Tauri windows, GTK, Quick Look, Spacedrop) still need the desktop binary.

Passed:
- Location added from the CLI appears in the sidebar; browsing and double-click into folders work.
- New Folder (Ctrl+Shift+N) creates a free "Untitled Folder" name and enters rename; rename to "Projetos" renamed on disk.
- Sort menu: choosing the active option flips the direction; arrow icon and order follow; direction persists per tab across reloads.
- Ctrl+H shows and hides dotfiles; F5 refreshes without reloading the page.
- Delete key opens the confirmation, the file moves to the trash, and the dialog closes in about 200 ms.

Bugs found and fixed in this session:
- `wing-server` panicked at startup (axum 0.8 route syntax).
- Web UI rendered blank without the private Spacebot repo.
- Directory names sorted case-sensitively in SQL.
- The delete dialog never closed after confirming.
- `useWaitForJob` missed events of fast jobs and waited 30 s.
- The overflow sort submenu showed a stale direction.
- `wing-cli file list` labelled folders as files and dropped extensions.

Open:
- WATCH-003: deleted files stay listed (index not updated by the watcher).
- Device shows as "Unknown Device" in an isolated instance; check whether a fresh install names the device.
- Grid, list, media, column views beyond grid; copy/move with progress; Inspector, Quick Preview, Job Manager windows; native Tauri run.

## Linux Runtime Session 2026-10-01 (second pass)

Native Tauri app (`tauri dev --no-watch`, `GDK_BACKEND=x11`) connected to an isolated daemon and the same checks through the web build.

Passed: deep links, grid/list/column/media views, sort by name and size (files and folders), thumbnails, Quick Preview (navigate, close), Inspector, copy and move with conflict dialog, Job Manager popover and screen, native Settings window.

Fixed in this pass: tab restore overriding explicit URLs; folders sorted by inode size; web build forgetting the library; locations stuck in "scanning"; desktop daemon built without ffmpeg/heif (no thumbnails); inspector showing "Unknown" scan state; content-addressed rows breaking copy/move/delete with duplicates; generic "Job failed" messages; names without extensions in Quick Preview and delete dialogs; jobs listed oldest first with 0s durations; "Unknown Device" on Linux; duplicate directory entries (WATCH-004).

Open: search and recents click-through, `Ctrl+number` view shortcuts are taken by the browser in the web build (fine in Tauri), Spacedrop (needs two devices), packaged bundle (TAURI-011).
