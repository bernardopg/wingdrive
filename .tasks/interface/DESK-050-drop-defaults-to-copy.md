---
id: DESK-050
title: File drops default to Copy on the same device in the desktop build
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer, dnd, bug]
last_updated: 2026-11-04
---

## Description

`DndProvider.dropFilesInto` guesses Move when the source and destination share a device slug, as Linux file
managers do, and Ctrl or Shift override the guess. In the desktop debug build under Xvfb on 2026-11-04 the
copy/move dialog opened with Copy for every drop, including a file dropped on a folder in the same pane,
with no modifier held (pointer events reported `ctrlKey` false). Found while checking DESK-016.

## Acceptance Criteria

- [ ] A drop within one device opens the dialog on Move, and Ctrl switches it to Copy
- [ ] Root cause recorded (slug mismatch, stale modifier state, or another path)
- [ ] Verified on the Linux desktop build
