---
id: DESK-000
title: "Epic: WingDrive as the default Linux file manager"
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M2
sprint: S05
tags: [epic, desktop, linux, file-manager]
last_updated: 2026-11-15
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
- [x] DESK-010 Startup latency and resident memory budget
- [x] DESK-011 System WebKit memory on Arch
- [x] DESK-012 Linux navigation compositing
- [x] DESK-013 i18n and theming follow-ups
- [x] DESK-014 Accurate directory sizes
- [x] DESK-015 Open a folder in a new window
- [x] DESK-016 Dual-pane Explorer
- [x] DESK-017 Close M2: AUR runtime evidence and a full working day
- [x] DESK-018 Copy To and Move To a chosen folder
- [x] DESK-019 Invert selection and select by pattern
- [x] DESK-020 On-demand file checksum in the Inspector
- [x] DESK-021 Search refinement bar
- [x] DESK-025 Drag files from the Explorer into other apps
- [x] DESK-026 Filter the current folder by name
- [x] DESK-027 Status bar with counts, selection size and free space
- [x] DESK-028 Ctrl+L edits the path bar
- [x] DESK-029 Folder tree and expandable list rows
- [x] DESK-030 Group items by kind, date or size
- [x] DESK-031 New file from the Templates folder
- [x] DESK-032 User-defined context menu actions
- [x] DESK-033 Act on system files as administrator
- [x] DESK-034 Terminal panel docked in the Explorer
- [x] DESK-035 Git status emblems on files
- [x] DESK-036 Open in a new tab from the sidebar and folders, and drop files on tabs
- [x] DESK-051 WingDrive as the file chooser for uploads and saves
- [x] DESK-097 A hidden login start stays in the tray, and "Start minimized in the tray" is a setting
- [x] DESK-090 to DESK-096 Bugs found in the S03 and S04 revalidation

## Acceptance Criteria

- [x] `xdg-mime default wingdrive.desktop inode/directory` makes `xdg-open <dir>` open WingDrive
- [x] Every child task is Done with runtime evidence from the AUR package
- [x] A full working day on the author's machine without falling back to another file manager

## Evidence

- DESK-001..009 Done in #135 with tests and runtime evidence in each task file; DESK-010..012 and
  DESK-014 Done since, each with its own evidence.
- With isolated `XDG_DATA_HOME`/`XDG_CONFIG_HOME` holding the shipped desktop entry, `xdg-mime default
  wingdrive.desktop inode/directory` wrote `inode/directory=wingdrive.desktop` and `gio mime inode/directory`
  reported `wingdrive.desktop` as the default, which is what `xdg-open <dir>` uses.
- DESK-017 (2026-11-05): the author used the AUR package for several days without going back to
  another file manager. Each child task records its own check on the AUR package.
- Closed on 2026-11-15: DESK-051 confirmed by the author with `wingdrive-bin` `3.3.1` (cold login,
  upload in Chrome), and DESK-097 (window shown at login) fixed and reproduced with a named
  instance. With every child Done, M2 is complete.
