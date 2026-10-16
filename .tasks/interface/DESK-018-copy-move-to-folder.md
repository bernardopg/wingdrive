---
id: DESK-018
title: Copy To and Move To a chosen folder
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, explorer, file-operations, linux]
last_updated: 2026-10-15
---

## Description

Dolphin and Nautilus offer "Copy To…" and "Move To…" in the context menu, which pick a destination
folder without opening a second window or dragging. WingDrive only copies and moves by drag, by
cut/paste, or across windows. The pieces exist: `useFileOperationDialog` runs copy and move with
conflict resolution, and the desktop already grants `dialog:allow-open` for a folder picker.

## Implementation Steps

1. Add "Copy To…" and "Move To…" to the Explorer context menu for a selection of local files.
2. Open the native folder picker, starting in the current folder; cancel does nothing.
3. Pass the selection and the chosen folder to `useFileOperationDialog` with `copy` or `move`.
4. Add the strings to `en` and `pt`.

## Acceptance Criteria

- [x] Both entries copy or move the selection into the picked folder, with the usual conflict dialog
- [x] Cancelling the picker changes nothing
- [x] The entries are hidden where the picker cannot work (web host, cloud paths)
- [ ] Verified on the Linux desktop build

## Evidence

- `useCopyMoveTo` opens the native folder picker and hands the selection to `useFileOperationDialog`,
  so copy and move get the existing conflict dialog and undo. The entries need `openDirectoryPickerDialog`
  (desktop only) and local physical sources, so the web host and cloud paths never show them.
- `copyMoveDestination.test.ts` covers the picked destination, a cancelled picker and non-local sources.
- Open: runtime check on the Linux desktop build.
