---
id: TAURI-008
title: Standard File Manager Keyboard Shortcuts
status: To Do
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

- [ ] `explorer.refresh`: F5 and Ctrl/Cmd+R refetch the current listing
- [ ] `explorer.newFolder`: Shift+Ctrl/Cmd+N creates a folder in the current directory and starts rename
- [ ] `explorer.toggleHiddenFiles`: Ctrl+H on Linux/Windows, Cmd+Shift+. on macOS
- [ ] Shortcuts do not fire while an input or rename field has focus
- [ ] Verified in the Linux desktop app

## Implementation Files

- `packages/interface/src/util/keybinds/registry.ts`
- `packages/interface/src/routes/explorer/hooks/useEmptySpaceContextMenu.ts`
