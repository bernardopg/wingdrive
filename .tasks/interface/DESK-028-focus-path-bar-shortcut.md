---
id: DESK-028
title: Ctrl+L edits the path bar
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard]
last_updated: 2026-10-27
---

## Description

Ctrl+L (Alt+D on Windows, Cmd+Shift+G on macOS Finder) turns the location bar into a text field in
every file manager. WingDrive's `PathBar` can be edited with the mouse, but no shortcut opens it.

## Implementation Steps

1. Register `explorer.focusPathBar` (Ctrl+L) and show it in the shortcuts list.
2. In `PathBar`, enter edit mode with the current folder's path selected so typing replaces it.
3. Expand a leading `~` to the home folder when submitting.

## Acceptance Criteria

- [ ] Ctrl+L opens the path field with the path selected
- [ ] Enter navigates, Escape restores the breadcrumbs
- [ ] Verified on the Linux desktop build

## Evidence

- Open.
