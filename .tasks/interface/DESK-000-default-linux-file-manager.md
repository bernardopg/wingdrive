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

- [x] DESK-001 XDG integration, command-line paths and single instance
- [x] DESK-002 `org.freedesktop.FileManager1` D-Bus service
- [x] DESK-003 Background mode in the system tray with autostart
- [x] DESK-004 Real "Open With" on Linux
- [x] DESK-005 "Open Terminal Here"
- [x] DESK-006 New empty file and symbolic link
- [x] DESK-007 Permissions and ownership in Properties
- [x] DESK-008 Compress and extract archives
- [x] DESK-009 Network locations and devices through GIO/gvfs
- [ ] DESK-010 Startup latency and resident memory budget

## Acceptance Criteria

- [x] `xdg-mime default wingdrive.desktop inode/directory` makes `xdg-open <dir>` open WingDrive
- [ ] Every child task is Done with runtime evidence from the AUR package
- [ ] A full working day on the author's machine without falling back to another file manager

## Evidence

- DESK-001..009 Done in #135 with tests and runtime evidence in each task file; DESK-010 in progress.
- With isolated `XDG_DATA_HOME`/`XDG_CONFIG_HOME` holding the shipped desktop entry, `xdg-mime default
  wingdrive.desktop inode/directory` wrote `inode/directory=wingdrive.desktop` and `gio mime inode/directory`
  reported `wingdrive.desktop` as the default, which is what `xdg-open <dir>` uses.
- Open: runtime evidence from the published AUR package and a full working day on the author's machine.
