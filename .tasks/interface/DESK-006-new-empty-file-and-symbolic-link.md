---
id: DESK-006
title: New empty file and symbolic link
status: Done
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

- [x] Core actions `files.createFile` and `files.createSymlink` with validation and conflict errors
- [x] UI: New File in the empty-space and folder menus with inline rename; Create Link for a selection
- [x] Watcher picks up the new entries in indexed locations; ephemeral views refresh

## Acceptance Criteria

- [x] New File creates `Untitled File` (unique name) and starts rename
- [x] Create Link creates `<name> (link)` pointing to the target
- [x] Integration tests for both actions

## Evidence

- `core/src/ops/files/create_entry/mod.rs`: `files.createFile` and `files.createSymlink`, local paths only, never
  replace an existing name. 3 core tests pass. TypeScript types regenerated.
- Desktop undo `undo_new_file` removes the entry only if it is the same inode and still empty; Rust test covers empty
  files, files that gained content (kept) and links.
- UI: New File in the empty-space and item menus and `Ctrl+Alt+N`; Create Link on a single item. Names avoid
  collisions using on-disk names with extensions (`report.txt (link)`); bun test for the name helper.
- Xvfb runtime: New File created `Untitled File`, selected it and opened rename; typing `todo.md` renamed it on disk.
  Undo from the menu removed an untouched new file. Create Link on `report.txt` made `report.txt (link)` pointing to
  it. With `Ctrl+Alt+N` the new file appeared within 1 s. In one earlier attempt, right after startup, the file was
  created but the listing had not shown it after 3 s.
- The empty-space menu now opens anywhere outside an item in grid and list views; before, inner wrappers swallowed
  the right click.
