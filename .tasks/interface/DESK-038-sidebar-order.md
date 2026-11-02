---
id: DESK-038
title: Sidebar item ordering
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, sidebar, accessibility]
last_updated: 2026-11-01
---

## Description

Let people reorder sidebar groups, shortcuts and dynamic items with a pointer or keyboard.
Dynamic item order stays local to the device and space group.

## Acceptance Criteria

- [x] Reorder stored shortcuts within their group or at the space root.
- [x] Place the folder tree among the space groups.
- [x] Reorder locations, volumes, devices, tags, sources and network entries within their lists.
- [x] Restore dynamic item order after reload, including items that temporarily disappear.
- [x] Keep navigation and file drop actions available alongside drag handles.
- [x] Cover ordering and list boundaries in the test suite.
- [ ] Complete the repository Definition of Done.

## Evidence

- `reorderItems.test.ts` covers group boundaries, missing items and saved order; it runs in CI.
- Chromium against an isolated daemon: moved volumes, root shortcuts, groups and the folder tree; order survived reload. Keyboard reordering and a file drop confirmation also worked.
- Tauri debug build on Xvfb: opened the terminal from the toolbar, ran a command, hid it and reopened the same shell with its output intact. On application restart the terminal stayed closed and no shell started until the button was pressed.
