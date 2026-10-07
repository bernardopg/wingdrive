---
id: DESK-001
title: XDG integration, command-line paths and single instance
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Critical
milestone: M2
sprint: S03
tags: [desktop, linux, xdg]
last_updated: 2026-10-07
---

## Description

The desktop entry declares no folder MIME type and `Exec` takes no argument, and the app ignores its command line. A second launch starts another full Tauri process. WingDrive must register for `inode/directory`, open the path or `file://` URI it is given, and route later launches to the running instance.

## Implementation Steps

- [ ] Desktop entry with `MimeType=inode/directory;` and `Exec=wingdrive %U` from the Tauri bundle config
- [ ] Parse paths and `file://` URIs from argv; a file selects it in its parent folder
- [ ] `tauri-plugin-single-instance` forwards argv to the running app, which opens a tab and focuses the window
- [ ] Frontend event that navigates the active tab to a physical path
- [ ] AUR package installs the desktop entry with the MIME type

## Acceptance Criteria

- [ ] `xdg-open ~/Downloads` opens WingDrive at that folder
- [ ] A second `wingdrive <dir>` opens a tab in the running window instead of a new process
- [ ] `wingdrive <file>` opens the parent folder with the file selected
