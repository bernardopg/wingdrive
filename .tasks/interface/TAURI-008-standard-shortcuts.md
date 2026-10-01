---
id: TAURI-008
title: Standard File Manager Keyboard Shortcuts
status: Done
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M1
sprint: S01
tags: [tauri, explorer, keybinds]
last_updated: 2026-10-01
---

## Description

The 2026-09-03 review found no keybinds for refresh, new folder, or hidden files. Add them through the existing keybind registry.

## Acceptance Criteria

- [x] `explorer.refresh`: F5 on Linux/Windows, Cmd+R on macOS (one combo per platform in the registry)
- [x] `explorer.newFolder`: Ctrl/Cmd+Shift+N creates a folder in the current directory and starts rename
- [x] `explorer.toggleHiddenFiles`: Ctrl+H on Linux/Windows, Cmd+Shift+. on macOS
- [x] Shortcuts do not fire while an input or rename field has focus (`useKeybind` default)
- [x] Verified in the Linux desktop app (web UI on the same interface, 2026-10-01; see TAURI-006)

## Notes

- New Folder picks the first free "Untitled Folder", "Untitled Folder 2", ... name. Before, a second New Folder in the same directory failed because the name already existed. The context menu uses the same `useCreateFolder` hook and now shows the shortcut.
- `nextFolderName` has a bun test, added to CI.

## Implementation Files

- `packages/interface/src/util/keybinds/registry.ts`
- `packages/interface/src/routes/explorer/hooks/useEmptySpaceContextMenu.ts`
