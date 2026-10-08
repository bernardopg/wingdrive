---
id: DESK-000
title: "Epic: WingDrive as the default Linux file manager"
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: Critical
milestone: M2
sprint: S03
tags: [epic, desktop, linux, file-manager]
last_updated: 2026-10-07
---

## Description

WingDrive must replace Dolphin, Nautilus or Yazi as the file manager the user opens every day. That needs desktop integration (`xdg-open` on folders, "Show in folder" from other apps), instant opening from a resident app in the tray, and the everyday operations a file manager is expected to have. Navigation stays graphical; a Yazi-style modal keyboard mode is out of scope.

## Implementation Steps

- [ ] DESK-001 XDG integration, command-line paths and single instance
- [ ] DESK-002 `org.freedesktop.FileManager1` D-Bus service
- [ ] DESK-003 Background mode in the system tray with autostart
- [ ] DESK-004 Real "Open With" on Linux
- [ ] DESK-005 "Open Terminal Here"
- [ ] DESK-006 New empty file and symbolic link
- [ ] DESK-007 Permissions and ownership in Properties
- [ ] DESK-008 Compress and extract archives
- [ ] DESK-009 Network locations and devices through GIO/gvfs
- [ ] DESK-010 Startup latency and resident memory budget

## Acceptance Criteria

- [ ] `xdg-mime default wingdrive.desktop inode/directory` makes `xdg-open <dir>` open WingDrive
- [ ] Every child task is Done with runtime evidence from the AUR package
- [ ] A full working day on the author's machine without falling back to another file manager
