---
id: DESK-003
title: Background mode in the system tray with autostart
status: To Do
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

- [ ] Tray icon with menu: Open WingDrive, New Window, Quit
- [ ] Closing the main window hides it while background mode is on; Quit exits and stops the owned daemon
- [ ] Setting "Keep running in the background" persisted in the app
- [ ] Setting "Start at login" writes or removes `~/.config/autostart/wingdrive.desktop` with `--hidden`
- [ ] `--hidden` starts without showing a window

## Acceptance Criteria

- [ ] Closing the window leaves the tray icon and the daemon running; reopening from the tray or `wingdrive <dir>` is instant
- [ ] Quit from the tray stops the owned daemon
- [ ] Start at login starts WingDrive hidden after a new session
