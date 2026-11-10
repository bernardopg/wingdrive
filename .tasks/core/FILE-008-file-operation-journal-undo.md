---
id: FILE-008
title: Job journal and undo for copy, overwrite and trash
status: In Progress
assignee: bernardopg
parent: FILE-000
priority: Medium
milestone: M3
sprint: S05
tags: [files, operations, undo, trash, jobs, upstream, port]
last_updated: 2026-11-10
related_tasks: [FILE-007, EXPL-003]
---

## Description

Split from item 7 of the 2026-10-14 upstream survey (upstream `bcc7337b0`). WingDrive's undo
(EXPL-003, `apps/tauri/src-tauri/src/undo.rs` plus `undoStore.ts`) covers
rename, same-volume move without overwrite and new empty folders. It cannot
undo a copy, a cross-volume move, an overwrite or a move to trash, because
nothing records what those jobs changed.

Record each file job's effects in the library database (created, moved,
trashed and where to, replaced and where the previous bytes went, removed),
then offer undo as a job that reverses the journal after checking nothing
changed since.

## Implementation Notes

- Journal table written by the job through its context, so resume and
  undo read the same record; the undo job journals itself.
- Overwrites in copy keep the previous file in a per-job stash on the same
  volume instead of deleting it; stash cleanup has a retention limit.
- Trash records the location the OS trash reported (the `trash` crate lists
  it on Linux); undo restores from there.
- Undo refuses, per item, when the current file no longer matches what the
  journal recorded; it reports which items it skipped.
- Moves the Tauri-side journal for rename and move into the core, so the CLI
  and web get undo too.

## Acceptance Criteria

- [x] Copy, move, overwrite and trash jobs write a journal entry per effect
- [x] Undo reverses each of them and refuses items changed since
- [x] Overwritten files are recoverable until the stash expires
- [ ] Existing rename/move/folder undo keeps working (current tests pass)
- [x] Docs in `docs/core/` updated

## Implementation

- `core/src/ops/files/journal.rs`: `Effect` (created, moved, replaced, trashed, merged), the `Seal`
  that undo checks, the per-job stash folder and the 7-day expiry. Table `file_journal` in the job
  database (`core/src/infra/job/database.rs`), which is per library and never synced.
- `FileCopyJob` and `DeleteJob` write the journal. A failed or cancelled overwrite puts the stashed
  file back at once.
- `core/src/ops/files/undo/`: `FileUndoJob`, action `files.undo`, query `files.undo_history`,
  output `JobOutput::FileUndo`.
- CLI: `wing file undo [JOB_ID]` and `wing file history`.
- Interface: `FileOperationModal` records every copy and move with the job id and undoes it through
  `files.undo`. This replaces the Linux-only inode undo it had for same-volume moves.

## Evidence

- 2026-11-10: `core/tests/file_undo_test.rs`, 14 cases with real jobs, in the Linux gate of
  `scripts/ci-rust.sh`: copy, overwrite, move (rename and copy-then-delete), rename, trash, a copy
  edited afterwards, an old place taken again, a merge, a failed overwrite, expiry, history and the
  action. Unit tests for the seal in `journal.rs`.
- Isolated daemon (`--instance undo8`): `wing file copy --overwrite` of two files over one existing
  file, `wing file history` showed `3 of 3`, `wing file undo` brought the previous content back and
  removed the copies and the stash folder; history then showed `0 of 3` and a second undo answered
  `Nothing to undo`.

- Web UI on an isolated `wing-server`, driven with headless Chrome: copied two files from `src`,
  pasted into `dest` over an existing `report.txt` choosing "Overwrite existing files", then Ctrl+Z.
  Disk after the paste: `.wing-undo-<job>`, `extra.txt`, `report.txt` = new. After Ctrl+Z:
  `report.txt` = previous only, and the toast "Undid copy". The Tauri shell was not driven.

## Open

- [ ] Existing rename/move/folder undo keeps working: the desktop still uses the Tauri commands for
      rename, new folder and new file. Moving those onto the core journal is not done, so the CLI
      and web get undo for copy, move, rename and trash only.
- The undo job does not journal itself (no redo).
- The seal cannot tell an edit that restores the same size within the same change-time tick; on
  Windows it uses modification time and no inode.
- A folder copied over an existing folder is a merge and is not undoable.
