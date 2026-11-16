---
id: TAURI-019
title: Windows daemon runs without a console window, with a tray icon
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S07
tags: [tauri, windows, daemon, tray]
related_tasks: [TAURI-012, TAURI-016, TAURI-017, DESK-003]
last_updated: 2026-11-15
---

## Description

Reported by the author on 2026-11-15. On Windows, `wing-daemon.exe` opens a terminal window, and
closing it stops WingDrive. On Linux the daemon is a systemd user service with no window.

`wing-daemon` is a console program. The desktop app registers it as the `WingDriveDaemon` scheduled
task (`schtasks`, `apps/tauri/src-tauri/src/main.rs`), and Windows gives a console program started
in the user's session its own console window. The Tauri app itself uses
`windows_subsystem = "windows"`, so only the daemon shows a console.

Expected behavior, like OneDrive, Google Drive, Dropbox or iCloud: the service runs in the background
with no window and a small notification-area icon. The icon opens WingDrive, shows the daemon state
and the background settings, and quits.

## Implementation Steps

- [ ] Start the daemon with no console: build `wing-daemon` for Windows as a GUI-subsystem program
      that logs to its log file, or start it with `CREATE_NO_WINDOW`; the scheduled task keeps
      working without a window either way.
- [ ] `wing daemon start` and `wing restart` on Windows start it the same way.
- [ ] The notification-area icon belongs to the WingDrive app in background mode (DESK-003), which on
      Windows keeps running hidden when "Keep running in the background" is on. Check that the icon,
      its menu and "Start at login" work there, and that the daemon keeps running when the window
      closes.
- [ ] Logs reach a file the user can open from the tray menu or Settings, since there is no console.

## Acceptance Criteria

- [ ] On a real Windows machine, logging in starts the daemon and WingDrive with no console window.
- [ ] Closing every WingDrive window leaves the daemon running and the tray icon in place.
- [ ] Quitting from the tray stops the app; the daemon follows the "Keep running" setting.
