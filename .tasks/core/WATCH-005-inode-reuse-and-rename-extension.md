---
id: WATCH-005
title: Watcher takes inode reuse for renames and keeps stale extensions
status: Done
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M1
sprint: S01
tags: [watcher, indexing, database, bug]
last_updated: 2026-10-05
---

## Description

Found during the TAURI-014 runtime session. Deleting `dest/report-copy.txt` and then creating `media.png` left the index with a 2-byte `media.txt` and no `media.png`. Two bugs combined:

1. ext4 hands a freed inode to the next new file at once. The persistent handler pairs a buffered Remove and a Create by inode alone, so it took the new file for a rename of the deleted one.
2. A rename only rewrote `name` from the file stem. The stored extension never changed, and directories with a dot ("release-v1.2") lost the part after the dot.

## Implementation Steps

- [x] Accept an inode match as a rename only when the file keeps the recorded size and modification time; otherwise emit Remove and Create
- [x] Share one name/extension rule between indexing and both move paths
- [x] Cover extension change, dotted directory rename and inode reuse in `fs_watcher_test`

## Acceptance Criteria

- [x] Deleting a file and creating another one that reuses its inode indexes the new file with its own name, extension and size
- [x] Renaming `a.txt` to `a.md` stores extension `md`
- [x] Renaming a directory to a dotted name stores the full name
- [x] `cargo test -p wing-core --test fs_watcher_test` passes

## Evidence (2026-10-05)

- `cargo test -p wing-core --test fs_watcher_test`: passes with the three new steps (extension change, dotted directory rename, inode reuse).
- Without the fix the same test fails: `Entry 'kind-change' with extension Some("md") not found; found [Some("txt")]`.
- `stored_name_tests` (2) pass; `cargo clippy -p wing-core -D warnings` clean.
