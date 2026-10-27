---
id: DESK-025
title: Drag files from the Explorer into other apps
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, explorer, dnd, linux]
last_updated: 2026-10-27
---

## Description

Dolphin, Nautilus, Thunar, Finder and Windows Explorer let you drag a file into a browser upload,
a chat window, an editor or another file manager. WingDrive only drags inside its own window: the
grid and list items use the in-app dnd-kit drag, and the native `startDrag` platform API
(`apps/tauri/src/platform.ts`) is used by `SizeView/SizeCircle.tsx` alone. The Rust drag
coordinator in `apps/tauri/src-tauri/src/drag/` only relays events between WingDrive windows; it
never starts an OS drag session.

## Implementation Steps

1. Start a real OS drag session from Tauri with the selected files as `text/uri-list` (GTK on Linux
   first; macOS and Windows through the same API when the library supports them).
2. Keep the in-app drag for moves into folders, the sidebar and tabs. Hand over to the OS drag when
   the pointer leaves the window, or start the OS drag directly when no in-app target applies.
3. Drag the whole selection when the dragged item is selected, otherwise only that item.
4. Only physical local paths can be dragged out; cloud and content paths stay in-app.
5. Tests for the item-to-URI mapping; one runtime check dropping into Firefox and Dolphin.

## Acceptance Criteria

- [ ] Dragging a file from the grid or list into another app hands it the file
- [ ] Dragging a multi-selection hands over every selected file
- [ ] In-app moves into folders, sidebar items and tabs still work
- [ ] Verified on the Linux desktop build

## Evidence

- Open.
