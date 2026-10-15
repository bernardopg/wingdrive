---
id: DESK-016
title: Dual-pane Explorer
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, linux]
last_updated: 2026-10-15
---

## Description

Dolphin's split view (F3) shows two folders side by side, which makes moving
and comparing files fast. WingDrive has no split view. This is the larger of
the two "two places at once" options; DESK-015 (a new window) covers most of
the need with far less work, so decide on this one after DESK-015 ships.

## Implementation Steps

1. Decide the model: two independent Explorer contexts in one window, each
   with its own path, history, selection and view settings.
2. Toggle with F3 and a toolbar button; the focused pane receives keyboard
   shortcuts and toolbar actions.
3. Drag and drop and copy/paste between panes use the existing file
   operations; "Copy/Move to other pane" goes in the context menu.
4. Persist the split state per tab.

## Acceptance Criteria

- [ ] F3 toggles a second pane with its own navigation
- [ ] Shortcuts act on the focused pane only
- [ ] Copy and move between panes work by drag and by context menu
- [ ] Verified on the packaged Linux build
