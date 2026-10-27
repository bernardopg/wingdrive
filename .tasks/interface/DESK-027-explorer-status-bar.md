---
id: DESK-027
title: Explorer status bar with item count, selection size and free space
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer]
last_updated: 2026-10-27
---

## Description

Every desktop file manager shows a status line under the view: how many items the folder holds,
what is selected and how large it is, and how much space is left on the drive. WingDrive shows
none of it; the Inspector covers a single file.

## Implementation Steps

1. A slim bar at the bottom of the Explorer reading `currentFiles` and the selection.
2. Without a selection: folder and file counts, and "M of N" while a filter is active.
3. With a selection: count of folders and files and the summed size of the selected files.
4. Free space of the volume holding the current folder, from `volumes.list` by the longest
   matching mount point on the same device.
5. Unit test the summary and volume matching; add the strings to `en` and `pt`.

## Acceptance Criteria

- [ ] Item counts match the visible listing, including while filtered
- [ ] Selection count and size update as the selection changes
- [ ] Free space shows for local folders and hides for virtual views
- [ ] Verified on the Linux desktop build

## Evidence

- Open.
