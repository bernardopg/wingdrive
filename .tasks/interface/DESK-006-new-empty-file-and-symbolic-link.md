---
id: DESK-006
title: New empty file and symbolic link
status: To Do
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [core, files, interface]
last_updated: 2026-10-07
---

## Description

Only folders can be created. A file manager must also create empty files and symbolic links.

## Implementation Steps

- [ ] Core actions `files.createFile` and `files.createSymlink` with validation and conflict errors
- [ ] UI: New File in the empty-space and folder menus with inline rename; Create Link for a selection
- [ ] Watcher picks up the new entries in indexed locations; ephemeral views refresh

## Acceptance Criteria

- [ ] New File creates `Untitled` (unique name) and starts rename
- [ ] Create Link creates `<name> (link)` pointing to the target
- [ ] Integration tests for both actions
