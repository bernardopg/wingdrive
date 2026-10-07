---
id: DESK-007
title: Permissions and ownership in Properties
status: To Do
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [core, files, interface, unix]
last_updated: 2026-10-07
---

## Description

Users need to see and change Unix permissions and, when allowed, the group. The Inspector shows no permission editor.

## Implementation Steps

- [ ] Query with mode, owner and group names for a path
- [ ] Core action `files.setPermissions` (mode, optional recursive for folders) and `files.setOwner` (group the user belongs to)
- [ ] Inspector section with read/write/execute checkboxes for owner, group, others and an octal field

## Acceptance Criteria

- [ ] Toggling execute on a script changes its mode on disk
- [ ] Errors such as EPERM are shown, not swallowed
- [ ] Integration tests for mode changes and the recursive variant
