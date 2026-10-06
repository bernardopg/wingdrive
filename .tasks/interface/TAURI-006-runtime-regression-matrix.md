---
id: TAURI-006
title: Add Tauri Runtime Regression Matrix
status: Done
assignee: unassigned
parent: TAURI-000
priority: High
sprint: S02
milestone: M1
tags: [tauri, testing, ci]
last_updated: 2026-10-06
---

## Description

Test the daemon-client path and native windows. A Vite build or typecheck alone cannot prove IPC, window labels, file opening, media playback, or platform dialogs.

## Acceptance Criteria

- [x] Start the packaged daemon and connect the main window (verified in TAURI-011 and the alpha.6 release smoke)
- [x] Browse a physical directory and open a file
- [x] Exercise grid, list, media, column, search, and recents
- [x] Open Settings, Inspector, Quick Preview, Job Manager (Spacedrop needs a second device)
- [x] Verify copy, rename, folder creation, delete confirmation, and job progress
- [x] Capture terminal state on Linux (macOS and Windows moved to TAURI-012)

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

## Linux Runtime Session 2026-10-06 (packaged bundle and two devices)

Production AppImage built from `main` plus #116 (INDEX-010), #117 (sync fixes) and #118 (BRAND-002),
run on Hyprland through XWayland (`GDK_BACKEND=x11`) with an isolated data dir and instance. The release
smoke script (`scripts/release/smoke.py`) also passed against the same bundle under Xvfb. A second device
(homesystem: Arch, 2 cores, 942 MB RAM, headless) ran only the packaged daemon (75 MB RSS) with the
bundle's libraries; it was paired over the LAN and driven through an SSH tunnel to its RPC port.

Passed:
- Fresh start creates the library; a location added from the CLI appears live with its folder card.
- Search: "This Folder" covers the subtree; "Library" scope; typing keeps the scope.
- Recents with image thumbnails.
- Grid, list, column, media and size views from the toolbar and from the overflow menu.
- Quick Preview of images, arrow navigation, Esc to close; selection follows.
- Rename (F2), external create picked up by the watcher, delete to the XDG trash, copy and move with the
  conflict dialog, results matching the disk.
- Settings as a native window; About shows Apache-2.0. Job Manager popover and full screen.
- Pairing with the second device by word code; both sides show Connected; automatic reconnection after
  restarting both daemons.
- Cross-device copy in both directions, byte-identical; a transfer to a path outside the receiver's
  locations fails with "Receiver rejected the transfer (PermissionDenied)".

Fixed in this pass (#119): local copy/move routed over the network when one path said `local` and the other
the device slug; rename bound to Enter only (now F2 off macOS); macOS ⌘ hints on Linux and false Cmd+1-5 view
hints; overflow menu under the TopBar/inspector and its flyouts unclickable; receiver rejection reported as
"transfer not found"; remote JobCompleted events dropped by the job activity decoder; CLI copy could not
address a paired device; Wingdrop reported success while sending nothing (now an explicit error; the View
menu that opened its sample-people window is debug-only).

Known limits, not blocking M1:
- Wingdrop itself is not implemented (`network.spacedrop.send` errors); device-to-device copy works through
  `files.copy`.
- A copy job whose every item failed still ends with status Completed; the output carries the failure counts
  and the UI shows them (TAURI-013), but remote and CLI consumers see "completed".
- The Wingbot window has no entry point without a configured runtime (TAURI-004).
- Synthetic keyboard input into XWayland needs `ydotool`; `xdotool type` and `wtype` do not reach the window
  on Hyprland. Note for future automated sessions.

