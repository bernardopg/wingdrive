---
id: WATCH-003
title: Watcher Does Not Sync Creates and Deletes on Linux
status: To Do
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

- [ ] Reproduce with debug logging (`RUST_LOG` must reach the daemon started by `sd-cli start`) and find where events stop
- [ ] Deleting a file in an indexed location removes its entry and the row disappears from the explorer
- [ ] Creating a file in an indexed location adds it without leaving the directory
- [ ] Holds after a daemon restart
- [ ] Integration test covers create and delete on Linux
