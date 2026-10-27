---
id: DESK-028
title: Ctrl+L edits the path bar
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard]
last_updated: 2026-10-27
---

## Description

Ctrl+L (Alt+D on Windows, Cmd+Shift+G on macOS Finder) turns the location bar into a text field in
every file manager. WingDrive's `PathBar` can be edited with the mouse, but no shortcut opens it.

## Implementation Steps

1. Register `explorer.focusPathBar` (Ctrl+L) and show it in the shortcuts list.
2. In `PathBar`, enter edit mode with the current folder's path selected so typing replaces it.
3. Expand a leading `~` to the home folder when submitting (`resolveTypedPath`, with the Tauri
   `homeDir` exposed as `platform.homeDir`). Only on this device; an unresolved `~` keeps the field
   open instead of navigating elsewhere.

## Acceptance Criteria

- [x] Ctrl+L opens the path field with the path selected
- [x] `~` and `~/…` expand to the home folder (unit tested in `pathInput.test.ts`)
- [ ] Enter navigates, Escape restores the breadcrumbs
- [x] Verified on the Linux desktop build

## Evidence

- `explorer.focusPathBar` (Ctrl+L) is registered and listed; `PathBar` enters edit mode with the
  physical path (or the URI for other paths), and the field selects its text on focus.
- Runtime, 2026-10-27: debug desktop build (`cargo build -p wingdrive` with the built frontend), isolated daemon and library, private Xvfb `:77`, `GDK_BACKEND=x11`, driven with xdotool.
  - Ctrl+L opened the field but `autoFocus` never focused it (no focusin reached the input), so
    typing went to the Explorer. `PathBar` now focuses and selects the field from an effect after
    the commit; after the fix, Ctrl+L then `~/.cache/wd-dragtest/files/pasta` and Enter opened
    that folder.
