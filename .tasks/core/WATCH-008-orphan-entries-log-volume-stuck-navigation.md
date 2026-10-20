---
id: WATCH-008
title: Removed locations leave their entries, logs grow without bound, tabs stop navigating
status: Done
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M2
sprint: S03
tags: [watcher, indexing, logging, cli, interface, bug]
last_updated: 2026-10-19
---

## Description

Found while checking the 2.0.1 update on the user's desktop and Moto G7.

1. The library kept 108,872 entries and 61,504 content identities with no location left. Removing `~/development` deleted its subtree with one `IN (...)` statement of 102,765 ids, above SQLite's 32,766 bound parameters; the error was dropped with `let _ =`, so the location went away and its entries stayed. Another 1,600 entries had no parent: the watcher created entries whose parent directory was not indexed (inside `target/`) instead of skipping them.
2. `~/.wingdrive/logs` reached 11 GB (10.9 GB in one day): every generated thumbnail, job progress tick, directory listing and volume list was an INFO line, and daily log files and per-job logs were never deleted.
3. `wing start` and `wing restart`, the commands the disconnected-daemon screen suggests, spawned a daemon next to the systemd unit, which stayed inactive and stopped restarting the daemon.
4. A temporary file deleted right after it was created logged an ERROR (`Failed to handle create ... No such file or directory`).
5. The Android release build sent core DEBUG logs to logcat (~550 lines per launch).
6. Desktop tabs sometimes stopped navigating: clicks registered, hover and right click worked, and opening another tab fixed it. Not reproduced live; a test reproduces the state the symptoms point to, where the tab router has moved but its render never commits, and remounting the router (what switching tabs does) recovers it.

## Implementation Steps

- [x] `DatabaseStorage::delete_entries_by_id` deletes in chunks of 900 and propagates errors; used by location removal and watcher deletes
- [x] Watcher creates under an unindexed parent return `ParentNotIndexed` and are skipped
- [x] Vanished paths are skipped at DEBUG in create and modify
- [x] Daily daemon logs keep 7 files; job logs older than 7 days are deleted when the library opens
- [x] Per-item and per-request INFO logs moved to DEBUG
- [x] `wing start` / `wing restart` go through the installed systemd unit
- [x] Mobile release builds log at `info`
- [x] `TabView` remounts a tab router whose location has not rendered 3 s after navigating, and logs a warning

## Acceptance Criteria

- [x] Removing a location deletes all its entries, however large
- [x] The watcher never stores an entry without a parent
- [x] Logs stay bounded
- [x] `wing restart` leaves `wingdrive-daemon.service` active
- [x] A stuck tab recovers without opening another tab

## Evidence (2026-10-19)

- `delete_subtree_tests::deletes_a_tree_larger_than_the_parameter_limit` (40,001 entries).
- `change_detection::handler::tests::skips_creates_under_an_unindexed_parent`, `vanished_paths_are_not_found`.
- `TabView.test.tsx` holds a navigation in a transition that never commits; it fails without the remount and passes with it.
- User library cleaned by hand (backup kept): `library.db` 109 MB to 5.4 MB, sidecars 92 MB removed, daemon logs 11 GB removed.
