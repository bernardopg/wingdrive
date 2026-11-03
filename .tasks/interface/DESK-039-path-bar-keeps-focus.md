---
id: DESK-039
title: Focus stayed in the path bar after Ctrl+L and Enter
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard, bug]
last_updated: 2026-11-02
---

## Description

After typing a path with Ctrl+L and pressing Enter, the Explorer navigated but the
keyboard focus stayed in a read-only input showing the folder name. The edit field and
the read-only field are both `<input>` elements in the same place in `PathBar.tsx`, so
React reused the focused node. Keys typed next (Ctrl+Shift+I, Ctrl+S, a tag key) went to
that input instead of the Explorer, and a second Ctrl+L did not reopen the editor.

Found during the packaged runtime check of the S04 tasks.

## Acceptance Criteria

- [x] After Enter, focus leaves the path bar
- [x] Explorer shortcuts work right after navigating with Ctrl+L
- [x] Verified on the packaged Linux build

## Evidence

- Fix: the edit input has its own `key`, so it unmounts on Enter or Escape instead of becoming the
  read-only field.
- Packaged build, 2026-11-02 (the DESK-018 setup): after Ctrl+L, a path and Enter, the active
  element is `BODY`, and Ctrl+Shift+I right away selected all 12 items of the folder. Before the
  fix the active element was the read-only `INPUT` and the shortcut did nothing.
