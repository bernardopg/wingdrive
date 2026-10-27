---
id: DESK-027
title: Explorer status bar with item count, selection size and free space
status: Done
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
- [x] Verified on the Linux desktop build

## Evidence

- `statusBar.ts` holds `summarizeFiles` and `volumeForPath` (longest mount point on the same device,
  mounted volumes only, extra `mount_points` included); `statusBar.test.ts` covers both.
- `ExplorerStatusBar` sits under every view except Size and reads `currentFiles`, the selection and
  `volumes.list` (`filter: All`). Without a selection it shows folder and file counts, or "M of N
  items shown" while filtered; with one, the count and summed size (folders add their content size
  when known). Free space shows only for physical paths whose device and volume resolve.
- Runtime, 2026-10-27: debug desktop build (`cargo build -p wingdrive` with the built frontend), isolated daemon and library, private Xvfb `:77`, `GDK_BACKEND=x11`, driven with xdotool.
  - Showed "1 folder · 2 files" and "338 GB free", "2 selected · 11 B" for a two-file selection,
    "1 of 2 items shown" while filtered, and "4 folders" in the device view (no free space there).
  - Found and fixed: list view's Size column read the raw `size` field (18 B for a folder whose grid
    card showed 2 B); it now uses `fileDisplaySize` like grid view. The stored folder total itself
    goes stale after a move into the folder, which is INDEX-014.
