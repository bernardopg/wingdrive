---
id: DESK-023
title: Folders listed with missing entries, slow first listings, list columns that do not fill the view
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [core, interface, indexing, bug, performance]
last_updated: 2026-10-23
---

## Description

Reported on the installed 2.1.0 desktop app: navigating between volumes and folders is very slow, `/home`
(the `home` volume) lists empty, and the list view leaves most of the central area blank with columns
capped at small widths.

Measured against the running daemon with `wing file list`:

1. `/home` returned 0 entries; `ls /home` has `bitter`. When `/` was scanned again,
   `EphemeralIndex::clear_directory_children` dropped `/home` (not browsed) but left the browsed
   `/home/bitter` in `path_index`. The next scan of `/home` skipped `bitter` as a duplicate, so it was never
   linked under the new `/home` node.
2. `/home/bitter/Downloads` returned 1 of 251 entries. Opening the `iso` volume
   (`Downloads/win10tg/iso`) created `Downloads` as an ancestor node holding only `win10tg`, and the listing
   served any cached node that had children, scanned or not.
3. `/usr/bin` (4,953 entries) and `/usr/lib` (7,321) index in about 400 ms, but the first listing waited
   250 ms and answered empty. The UI then refetched on every event batch, each refetch answered empty while the
   scan ran, and polling stopped after 8 updates.
4. An empty folder (`/mnt/wtgesp`) gets no node of its own, so every listing scanned it again: the daemon log
   showed a new indexer job every 750 ms while the UI polled.

## Implementation Steps

- [x] `clear_directory_children` keeps child folders that lead to a browsed folder and removes the whole subtree
      of every child it drops
- [x] `files.directory_listing` serves the cache only for a folder that was itself scanned; a scanned folder
      without a node lists as empty
- [x] A listing that finds the scan already running waits for it, like the listing that started it
- [x] First-listing wait raised from 250 ms to 2 s
- [x] List view: the name column fills the width the other columns leave until the user resizes it; rows and
      header span the whole view; no maximum widths; double-click on a resize handle restores the default;
      widths persist per tab

## Acceptance Criteria

- [x] Unit tests: rescanning a parent keeps the path to a browsed folder; dropped subtrees can be added again
- [x] Isolated daemon: `/home`, `Downloads` after the `iso` path, `/usr/bin` on the first call and an empty
      folder list correctly
- [x] Web UI: list spans the view, columns resize past the old caps, widths survive navigation
- [ ] Installed Linux package: the reported `/home`, volume and list view cases on the user's machine

## Evidence

- `cargo test -p wing-core --lib ephemeral::index`: 5 passed, including the two new regressions.
- Debug daemon with an isolated data dir and a tree mirroring the report (`home/bitter/Downloads` with 250 files
  and `win10tg/iso`, a 6,000-file folder, an empty folder), each call through `wing file list`:
  first listing of the 6,000-file folder returned 6,000 (1.5 s, debug build); `iso` 1, then `Downloads` 251,
  `bitter` 1, the root 3, then `home` 1 and `Downloads` 251 again. The installed 2.1.0 daemon had answered
  `/usr/bin` empty on the first call, `Downloads` with 1 entry and `/home` with 0. Three more listings of the empty folder dispatched no new indexer job (one in the log in total).
- `wing-server` web UI in Chrome at 1906 px: in list view the header and rows were 1406 px, the container
  width, with Name at 944 px. Dragging Size by 200 px gave 280 px (old cap 120) and Name shrank to keep the row
  full; dragging Name fixed it at 444 px. Widths were in the tab's saved state, survived a page reload and a
  folder change, and double-clicking the Name handle returned it to filling (744 px).
- 2026-10-23, release daemon from `main` (instance with its own data dir, raw RPC, `limit: null`): first listing
  of `/usr/bin` 4,925 entries in 228 ms and `/usr/lib` 7,320 in 310 ms, then 60 ms and 100 ms from the cache;
  `/home` 1, `/home/bitter` 41, `Downloads` 250, an empty folder 0, each 3 to 6 ms from the cache. The installed
  2.1.0 daemon answered `/usr/bin` and `/usr/lib` with 0 entries on the same calls.
- Not yet checked in the installed package. The installed 2.1.0 daemon keeps its in-memory index until it
  restarts, so `systemctl --user restart wingdrive-daemon` clears a stale `/home` until the fix ships.
