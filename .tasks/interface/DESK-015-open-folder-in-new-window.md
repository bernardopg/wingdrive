---
id: DESK-015
title: Open a folder in a new window
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer, windows, linux]
last_updated: 2026-10-15
---

## Description

Dolphin, Nautilus and Thunar can open a folder in a second window to work on
two places side by side. WingDrive has tabs but no way to start a window from
a folder, although the Tauri host already defines a multi-instance
`WingDriveWindow::Explorer { library_id, path }` that nothing in the
interface opens.

## Implementation Steps

1. Add "Open in New Window" to the folder context menu and a keybind
   (Ctrl+N on Linux and Windows, Cmd+N on macOS, matching other file managers).
2. Open the existing `Explorer` window kind through `windows::show_window`,
   with the folder's path and the current library.
3. Make the Explorer window route straight to that folder, sharing the tab and
   view state model of the main window.
4. Translate the new strings (en, pt) and add them to the keybind catalog.

## Acceptance Criteria

- [ ] Context menu and keybind open the folder in a new window
- [ ] Several Explorer windows can stay open at once, each on its own path
- [ ] Closing an Explorer window does not quit the app or stop the daemon
- [ ] Verified on the packaged Linux build
