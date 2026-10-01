---
id: WATCH-004
title: New Directories Indexed Twice by the Watcher
status: Done
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M1
sprint: S01
tags: [watcher, indexing, database, bug]
last_updated: 2026-10-01
---

## Description

Found while closing WATCH-003. Creating a directory inside an indexed location produces two `entries` rows for it. `handle_create` inserts the directory, then `handle_new_directory` dispatches an `IndexerJob` rooted at that directory, and the job inserts the root again. The unique index `(parent_id, name, extension)` does not stop it because directories have a NULL extension and SQLite treats NULLs as distinct.

`core/tests/fs_watcher_test.rs` (`test_location_watcher`) fails on this at `main` before and after WATCH-003: "Entry count mismatch: expected 4, got 5 ... projects (DIR), projects (DIR)".

## Acceptance Criteria

- [x] A new directory yields exactly one entry
- [x] Uniqueness also holds for directories: migration `m20261001_000001_unique_directory_entries` merges existing duplicates (children, collections, sidecars, locations, user metadata move to the oldest row) and adds `idx_entries_unique_dir`; the change handler treats a lost insert race as "already exists"
- [x] `test_location_watcher` passes
- [x] `ephemeral_watcher_test` passes

## Root Cause

Not the sub-path indexer: two watcher Create events for the same folder raced between `find_by_path` and the insert, and the unique index only covered files. Both watcher tests also failed because their fake trash lived in `/tmp`, a separate tmpfs, so `rename(2)` returned EXDEV.
