---
id: DESK-008
title: Compress and extract archives
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [core, jobs, interface]
last_updated: 2026-10-07
---

## Description

Compress and Extract are standard file manager actions. They must run as resumable-safe jobs with progress.

## Implementation Steps

- [x] Job `archive.compress` to zip and tar.gz/tar.xz/tar.zst
- [x] Job `archive.extract` for zip, tar, tar.gz, tar.bz2, tar.xz, tar.zst, 7z
- [x] Reject paths that escape the destination (zip slip) and symlinks pointing outside
- [x] Context menu: Compress..., Extract Here, Extract To...
- [x] Progress and cancellation through the job system

## Acceptance Criteria

- [x] Round-trip tests for each format
- [x] Zip-slip archive is rejected with an error
- [x] Extracting a 1 GB archive shows progress and can be cancelled

## Evidence

- `core/src/ops/archive/`: `archive.compress` (zip, tar, tar.gz, tar.bz2, tar.xz, tar.zst) and `archive.extract`
  (those plus 7z). Writes go to a hidden temporary path and are renamed into place without replacing anything;
  a single top-level entry is extracted directly, several go into a folder named after the archive.
- Extraction rejects zip slip, absolute or escaping links, writes through earlier links, and skips device files.
  11 core tests: round trip for every writable format, 7z extraction, zip slip, escaping link, write through inner
  link, cancellation before and during a copy, name helpers.
- UI: Compress submenu with five formats, Extract Here and Extract To... on archives; toasts bracket the job.
- Xvfb runtime: compressing `project` made `project.tar.gz` (verified with `tar tvzf`) and the listing updated
  live; Extract Here on a two-entry `bundle.zip` made `bundle/` with both files.
- 1 GB file to `.tar.xz`: the Job Manager showed live progress; Cancel from the popover showed
  "Compressing cancelled", removed the partial file, and the daemon went idle. Cancel from `wing job cancel` did the same.

Fixed along the way, each with a test:
- Listings scoped to the `local` device placeholder never received live events (`infra/event/mod.rs`).
- A device slug changed through `device.update` never reached library device rows (`library/manager.rs`).
- Jobs left running by a previous process stayed "running" forever; they are now paused or failed on open.
- The sidebar Job Manager memo ignored the job list, freezing it on its first snapshot; progress for a job not
  yet listed now triggers a refetch.
- A cancelled archive job spun at 100% CPU because the cancel error used `ErrorKind::Interrupted`.
