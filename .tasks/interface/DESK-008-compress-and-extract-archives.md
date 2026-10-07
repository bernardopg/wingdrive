---
id: DESK-008
title: Compress and extract archives
status: To Do
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

- [ ] Job `archive.compress` to zip and tar.gz/tar.xz/tar.zst
- [ ] Job `archive.extract` for zip, tar, tar.gz, tar.bz2, tar.xz, tar.zst, 7z
- [ ] Reject paths that escape the destination (zip slip) and symlinks pointing outside
- [ ] Context menu: Compress..., Extract Here, Extract To...
- [ ] Progress and cancellation through the job system

## Acceptance Criteria

- [ ] Round-trip tests for each format
- [ ] Zip-slip archive is rejected with an error
- [ ] Extracting a 1 GB archive shows progress and can be cancelled
