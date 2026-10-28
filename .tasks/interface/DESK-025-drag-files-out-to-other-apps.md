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

- [x] Dragging a file from the grid or list into another app hands it the file (code path in place)
- [x] Dragging a multi-selection hands over every selected file (code path in place)
- [x] In-app moves into folders, sidebar items and tabs still work (the handoff only fires at the window edge)
- [x] Verified on the Linux desktop build under X11, dropping into a GTK `text/uri-list` target
- [ ] Verified under native Wayland (GDK_BACKEND=wayland) by dropping into Firefox or Dolphin

## Evidence

- `externalDragPaths` resolves the dragged item, or the selection when the item is in it, to local
  absolute paths; a selection holding any remote, cloud, content or virtual entry is not dragged
  out. `pointerLeftViewport` decides the handoff. Both are unit tested.
- The `drag` crate (CrabNebula) was not used: its GTK backend builds `file://` URIs without
  escaping, which breaks names with `#`, `%` or spaces in strict targets. `file_drag.rs` starts the
  GTK drag itself (`drag_begin_with_coordinates`, `text/uri-list`, copy/move/link) and encodes URIs
  with the clipboard's `file_uris` (`Url::from_file_path`), already tested for spaces, `#` and
  accents. The target flag is `OTHER_APP`, so a drop back on WingDrive goes through its own dnd.
- `DndProvider.onDragMove`: when an `explorer-file` drag reaches the window edge, it cancels the
  dnd-kit drag (Escape at the document, not bubbling) and calls `platform.startFileDrag`, which
  only the Linux Tauri platform provides. `onDragCancel` now clears the overlay, which a cancelled
  drag used to leave behind.
- macOS and Windows: the command returns "not available on this platform yet"; the platform does
  not expose `startFileDrag` there, so the in-app drag is unchanged. Covered by TAURI-012.
- `cargo check -p wingdrive`, `bun run typecheck` and `bun test src/routes/explorer` (63 pass) green.
- Runtime, 2026-10-27: debug desktop build (`cargo build -p wingdrive` with the built frontend), isolated daemon and library, private Xvfb `:77`, `GDK_BACKEND=x11`, driven with xdotool.
  - One file dragged past the window edge reached a GTK drop target as
    `file:///…/a%20b%23c.txt` (copy action); a two-file selection arrived as both URIs, with
    `caf%C3%A9%20100%25.txt` encoded correctly.
  - Dragging a row into a folder row still opened the File Operation dialog, and Move moved the file.
  - Found and fixed: dnd-kit auto-scroll slid wide list views sideways on the way to the edge,
    hiding the Name column. `holdScrollLeft` in `DndProvider` keeps the horizontal scroll of the
    dragged item's containers during file drags, releasing one frame after the drag ends; vertical
    auto-scroll is unchanged.
- Open: native Wayland. Not automated, since the author's Hyprland session was in use and no nested
  compositor is installed. GTK takes the drag serial from the seat's implicit grab, which the held
  button provides, so it is expected to work; check it by hand.
