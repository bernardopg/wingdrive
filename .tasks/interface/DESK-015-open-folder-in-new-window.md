---
id: DESK-015
title: Open a folder in a new window
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer, windows, linux]
last_updated: 2026-10-16
sprint: S04
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

## Implementation Notes

- The `Explorer` window loads the full shell at `/explorer?path=<WingPath JSON>`,
  built in Rust to match `explorerUrlForDirectory` (unit-tested against
  `encodeURIComponent`). Tabs persist per window label, so windows do not
  overwrite each other. `explorer-*` joins the default capability so the
  window can listen to daemon events.
- Ctrl+N (Cmd+N on macOS) opens the folder being browsed; the folder context
  menu opens the clicked folder.

## Acceptance Criteria

- [x] Context menu and keybind open the folder in a new window (#217; runtime check in DESK-017)
- [ ] Several Explorer windows can stay open at once, each on its own path
- [ ] Closing an Explorer window does not quit the app or stop the daemon
- [ ] Verified on the packaged Linux build
