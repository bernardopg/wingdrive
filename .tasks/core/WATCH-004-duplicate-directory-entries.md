---
id: WATCH-004
title: New Directories Indexed Twice by the Watcher
status: To Do
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M1
tags: [watcher, indexing, database, bug]
last_updated: 2026-10-01
---

## Description

Found while closing WATCH-003. Creating a directory inside an indexed location produces two `entries` rows for it. `handle_create` inserts the directory, then `handle_new_directory` dispatches an `IndexerJob` rooted at that directory, and the job inserts the root again. The unique index `(parent_id, name, extension)` does not stop it because directories have a NULL extension and SQLite treats NULLs as distinct.

`core/tests/fs_watcher_test.rs` (`test_location_watcher`) fails on this at `main` before and after WATCH-003: "Entry count mismatch: expected 4, got 5 ... projects (DIR), projects (DIR)".

## Acceptance Criteria

- [ ] A new directory yields exactly one entry (reuse the existing root entry in the sub-path indexer job)
- [ ] Uniqueness also holds for directories (for example `COALESCE(extension, '')` in the unique index, with a migration that merges existing duplicates)
- [ ] `test_location_watcher` passes
- [ ] `ephemeral_watcher_test` failure investigated (also fails at `main`)
