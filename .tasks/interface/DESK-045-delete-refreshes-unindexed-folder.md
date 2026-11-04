---
id: DESK-045
title: Deleted files stayed in the listing of folders outside a location
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, explorer, core, bug]
last_updated: 2026-11-03
---

## Description

Deleting from a folder that is not a location (served from the ephemeral index) left the rows on
screen. The explorer refetches as soon as the delete job completes, but the ephemeral index was only
updated later by the watcher. The daemon log of the author's session shows the result: 185 files
trashed in `~/Downloads` at 20:52:04, then a second delete of two of them at 20:52:12 failing with
`Target does not exist`.

## Acceptance Criteria

- [x] Deleted files leave the listing of an unindexed folder right away
- [x] A large delete (200 files) leaves no stale rows

## Evidence

- `DeleteJob` drops the deleted local paths from the ephemeral index through `ephemeral::responder::apply_batch` (the watcher's own handler) before it reports completion.
- Web UI on an isolated daemon, 2026-11-03: 201 files selected in an unindexed folder and trashed; the listing showed 0 rows 4 s later. Single deletes in an indexed location and an unindexed folder also left the listing.
- Desktop debug build, 2026-11-03: `target/debug/WingDrive` under Xvfb with a private D-Bus session, isolated data dir, instance, XDG_DATA_HOME and XDG_RUNTIME_DIR, driven through WebKitWebDriver with real key events. 150 files in an unindexed folder: Ctrl+A, Delete, confirm; 0 rows after 4 s and the folder empty on disk.
