---
id: DESK-001
title: XDG integration, command-line paths and single instance
status: Done
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

- [x] Desktop entry with `MimeType=inode/directory;` and `Exec=wingdrive %U` from the Tauri bundle config
- [x] Parse paths and `file://` URIs from argv; a file selects it in its parent folder
- [x] `tauri-plugin-single-instance` forwards argv to the running app, which opens a tab and focuses the window
- [x] Frontend event that navigates the active tab to a physical path
- [x] AUR package installs the desktop entry with the MIME type

## Acceptance Criteria

- [x] `xdg-open ~/Downloads` opens WingDrive at that folder
- [x] A second `wingdrive <dir>` opens a tab in the running window instead of a new process
- [x] `wingdrive <file>` opens the parent folder with the file selected

## Evidence

- `apps/tauri/src-tauri/src/launch.rs` parses folders, files, `file://` URIs and `--hidden`; 3 unit tests pass.
- `apps/tauri/src-tauri/wingdrive.desktop` is the deb/AppImage desktop template; `scripts/release/aur.py` writes the
  same entry. `desktop-file-validate` passes on both.
- Xvfb runtime, isolated instance: `WingDrive beta/target.txt` opened `beta`, scrolled to and selected `target.txt`.
- Xvfb runtime, default instance: with WingDrive open on `alpha`, a second `WingDrive beta/target.txt` exited in 52 ms;
  the running window gained a `beta` tab with `target.txt` selected, and only one WingDrive process remained.
- Named instances (`WINGDRIVE_INSTANCE`) skip single instance so isolated runs never forward to the user's app.
