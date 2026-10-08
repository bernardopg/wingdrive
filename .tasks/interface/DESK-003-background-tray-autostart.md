---
id: DESK-003
title: Background mode in the system tray with autostart
status: Done
assignee: bernardopg
parent: DESK-000
priority: Critical
milestone: M2
sprint: S03
tags: [desktop, tray, daemon]
last_updated: 2026-10-07
---

## Description

Opening WingDrive costs a cold daemon and WebKit start each time. The app should stay resident: closing the last window hides it to a tray icon, the tray menu opens windows and quits, and an autostart option starts it hidden at login.

## Implementation Steps

- [x] Tray icon with menu: Open WingDrive, New Window, Quit
- [x] Closing the main window hides it while background mode is on; Quit exits and stops the owned daemon
- [x] Setting "Keep running in the background" persisted in the app
- [x] Setting "Start at login" writes or removes `~/.config/autostart/wingdrive.desktop` with `--hidden`
- [x] `--hidden` starts without showing a window

## Acceptance Criteria

- [x] Closing the window leaves the tray icon and the daemon running; reopening from the tray or `wingdrive <dir>` is instant
- [x] Quit from the tray stops the owned daemon
- [x] Start at login starts WingDrive hidden after a new session

## Evidence

- `apps/tauri/src-tauri/src/background.rs`: tray (Open, New Tab, Quit), settings persisted in `desktop_settings.json`,
  XDG autostart entry with `--hidden` (AppImage uses `$APPIMAGE` as launcher). 4 unit tests pass, including writing and
  removing the autostart file.
- Settings > General has "Keep running in the background" (default on) and "Start at login".
- Hyprland session: the tray item registered with the StatusNotifierWatcher and its dbusmenu listed Open WingDrive,
  New Tab and Quit WingDrive. Closing the window left the process running with no mapped window; a second
  `WingDrive beta` mapped it again in 204 ms (debug build). Quit from the tray menu exited the app and logged
  `App exiting, leaving existing daemon running` for a daemon it did not start.
- Xvfb: `WingDrive --hidden` loaded the frontend with no visible window.
- AUR depends on `libayatana-appindicator`; deb on `libayatana-appindicator3-1`.
