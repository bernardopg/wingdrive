---
id: FILE-008
title: Job journal and undo for copy, overwrite and trash
status: To Do
assignee: bernardopg
parent: FILE-000
priority: Medium
milestone: M3
tags: [files, operations, undo, trash, jobs, upstream, port]
last_updated: 2026-10-16
related_tasks: [FORK-006, FILE-007, EXPL-003]
---

## Description

Split from FORK-006 item 7 (upstream `bcc7337b0`). WingDrive's undo
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

- [ ] Copy, move, overwrite and trash jobs write a journal entry per effect
- [ ] Undo reverses each of them and refuses items changed since
- [ ] Overwritten files are recoverable until the stash expires
- [ ] Existing rename/move/folder undo keeps working (current tests pass)
- [ ] Docs in `docs/core/` updated
