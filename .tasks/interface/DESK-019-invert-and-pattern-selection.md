---
id: DESK-019
title: Invert selection and select by pattern
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, explorer, selection, keyboard]
last_updated: 2026-10-15
---

## Description

Explorer has Select All but no way to invert the selection or select by name pattern, which
Dolphin (Ctrl+Shift+A / Ctrl+Shift+I) and Nautilus (Ctrl+S, "Select Items Matching") both offer for
bulk work in large folders. Selection already lives in `SelectionContext` with `selectAll`.

## Implementation Steps

1. Add `invertSelection` next to `selectAll` in the selection context, over the loaded listing.
2. Bind Ctrl+Shift+I in `useExplorerKeyboard` and add the action to the Edit menu or toolbar menu.
3. Add "Select by Pattern…" (Ctrl+S): a small dialog with a glob such as `*.jpg`, case-insensitive,
   that replaces the selection with the matching names.
4. Unit test the glob matcher and the inversion; add the strings to `en` and `pt`.

## Acceptance Criteria

- [ ] Ctrl+Shift+I inverts the selection in grid, list and column views
- [ ] Ctrl+S selects every item whose name matches the glob
- [ ] Unit tests cover the matcher and inversion
- [ ] Verified on the Linux desktop build
