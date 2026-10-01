---
id: WATCH-003
title: Watcher Does Not Sync Creates and Deletes on Linux
status: Done
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M1
sprint: S01
tags: [watcher, indexing, linux, bug]
last_updated: 2026-10-01
---

## Description

Found during TAURI-006 on 2026-10-01. In an indexed location on Linux, files deleted from disk (through the UI trash, the CLI, or `rm`) stay in `library.db` and keep appearing in the explorer, even after a page reload. After a daemon restart, a file created with `echo > file` was not indexed within 6 seconds either. The delete job does not touch the index, so the explorer depends on the watcher to drop the rows. Every delete therefore leaves a ghost row, which is a P0 file-manager bug.

## Evidence

- Location `play` registered: "Location worker started for da68b27f..." in the daemon log, after both starts.
- `entries` still holds `external`, `external2`, `del-me` after the files were trashed; `w2` created after restart never appeared.
- No watcher or remove lines in the log for those files at `info` level.
- `cargo test -p sd-fs-watcher` passes (18 tests), so the gap is between the inotify backend and the persistent handler, or in the handler.

## Acceptance Criteria

- [x] Reproduce with debug logging and find where events stop
- [x] Deleting a file in an indexed location removes its entry (rm, move out, and trash)
- [x] Creating a file in an indexed location adds it without leaving the directory (20 concurrent files: 20/20)
- [x] Holds after a daemon restart
- [x] Tests: `sd-fs-watcher` unit tests for move out, move in, and paired rename; `indexing_responder_reindex_test` (folder moved into a location) now passes

## Root Cause

- Plain `rm` already worked. Trash and "move out of the location" are renames to a path outside the watched tree. inotify reports only the source half, and the Linux handler buffered it as a modify, which the change handler skipped because the path no longer existed. The entry stayed in the index forever.
- The reverse case (moving a folder in from outside) was buffered as a modify of an unknown path and skipped too, so moved-in content was never indexed.
- Concurrent creates sometimes failed with SQLite `database is locked` (SQLITE_BUSY_SNAPSHOT, code 517), which `busy_timeout` does not wait on, and the file was silently left out of the index.
- The "created after restart was not indexed" observation was an artifact of the investigation: a `sqlite3 database.db` call created an empty `database.db`, and the daemon refuses to load a library that has both `database.db` and `library.db`.

## Fix

- `crates/fs-watcher/src/platform/linux.rs`: unpaired rename halves wait one tick. A source that no paired rename claims and whose path is gone becomes `Remove`. A target that exists becomes `Create`. Paired renames cancel both, so in-tree renames keep their entry and UUID.
- `core/src/ops/indexing/change_detection/handler.rs`: change handlers retry with backoff on "database is locked/busy"; the handlers are idempotent.
- `core/tests/helpers/indexing_harness.rs`: the core data directory moved to `<test_root>/data`. Locations were nested in the data directory, which the watcher ignores, so the watcher integration tests never saw events.

## Follow-up

WATCH-004: new directories get two entries (pre-existing, fails `test_location_watcher` at `main`).
