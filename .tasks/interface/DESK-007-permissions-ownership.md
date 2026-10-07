---
id: DESK-007
title: Permissions and ownership in Properties
status: Done
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

- [x] Query with mode, owner and group names for a path
- [x] Core action `files.setPermissions` (mode, optional recursive for folders) and `files.setGroup` (a group the user belongs to; changing the owning user needs root)
- [x] Inspector section with read/write/execute checkboxes for owner, group, others and an octal field

## Acceptance Criteria

- [x] Toggling execute on a script changes its mode on disk
- [x] Errors such as EPERM are shown, not swallowed
- [x] Integration tests for mode changes and the recursive variant

## Evidence

- `core/src/ops/files/permissions/mod.rs`: `files.permissions` query (mode, owner, group, groups available to the
  user), `files.setPermissions` (recursive uses `chmod -R` `X` semantics and never follows links), `files.setGroup`.
  3 core tests: exact mode, recursive tree with a document, a tool and a link to an outside file, name lookups.
- Inspector Permissions section: owner, group selector, read/write/execute grid, octal field, "Apply to enclosed
  items" for folders, read-only note when the user is not the owner. bun test covers octal parsing.
- Xvfb runtime on `run.sh` (644): ticking Owner Execute changed it to 744 on disk; typing 750 in Octal changed it to
  750. Failures surface as toasts with the OS error.
