---
id: FORK-006
title: Port the next upstream fixes from the 2026-10-14 survey
status: Done
assignee: bernardopg
parent: FORK-002
priority: Medium
milestone: M2
sprint: S04
tags: [fork, upstream, port]
last_updated: 2026-10-16
---

## Description

The 2026-10-14 survey (`docs/design/UPSTREAM_PORT_SURVEY.md`) classified every
upstream commit since the fork point and ported the cheap, self-contained fixes
under FORK-005. This task covers the remaining PORT candidates, ranked by value
over effort. Keep authorship on cherry-picks, credit upstream in adapted ports,
and keep WingDrive's version wherever it already fixed the same bug.

## Implementation Steps

1. `393158c23`: `stop_watching` waits for the watcher loop to exit, closing the
   same race at shutdown that FORK-005 fixed at library creation. Small.
   Done 2026-10-15 with its regression test
   (`stopping_the_watcher_inside_its_debounce_reopens_nothing`).
2. `cedc8d6f7` (monitor part only): mark volumes that disappeared as offline, so
   an unplugged USB drive stops showing as online. Medium.
   Done 2026-10-15: the monitor marks this device's tracked, non-network volumes
   offline when detection no longer returns them (`volume_offline_test`).
3. `4d18db0ab` + `bf1ca21a1`: bound thumbnail decoding by available memory and
   downscale video frames, reimplemented in WingDrive's thumbnail job. Medium.
   Done 2026-10-16: the thumbnail job ignored `max_concurrent` and decoded a
   whole 50-entry batch at once; it now runs at most `max_concurrent` (4)
   generations, lowered to one per free GiB (`decode_workers`). `bf1ca21a1` is
   ALREADY-HAVE: WingDrive's in-process filter graph has no `thumbnail=30`
   frame window and the decoder runs single-threaded by default.
4. `12cdfa4be`: ship icons as WebP (196 PNGs today) and keep source maps out of
   the bundle to shrink the AppImage. Medium.
   Done 2026-10-16: the 201 PNGs matched upstream byte for byte and are now
   the upstream WebP files, imported as `.webp` by interface, ts-client and
   mobile. `wing-server` embedded about 18 MB of web source maps; rust-embed
   now excludes `*.map` (`source_maps_are_not_embedded`). The Tauri build
   already skips source maps outside debug, so its vite config is unchanged.
5. `15d7df180` (move part) and `5bc474e72` (sync part): confirm they apply to
   WingDrive's code before porting. Small to medium.
   Move part done 2026-10-16: it applied. The copy job deleted a moved source
   only when the volume manager said "different volume", so a streaming move
   on one volume (or a job without a volume manager) left the file in both
   places and reported success. It now checks the filesystem
   (`move_streaming_test`, fails without the fix). Sync part done 2026-10-16:
   it applied too. A sync request for a library this device no longer holds
   was logged as an error and left unanswered, so the peer timed out and
   marked itself caught up. The multiplexer now replies `SyncMessage::Error`
   (one warning per connection) and backfill turns that reply into a failure.
6. `92dea58a4`: CI gate that fails when generated TypeScript types are stale.
   Medium.
   Done 2026-10-15 as a step of `scripts/ci-rust.sh full`, so CI and
   `just ci-local` share it; the Tauri schema half is not ported.
7. `bcc7337b0`, `3cd44f2da`: copy preflight and undo, as WingDrive features.
   Large; split into their own tasks when started.
   Split 2026-10-16: FILE-007 (preflight plan for copy, move and delete) and
   FILE-008 (job journal and undo for copy, overwrite and trash), both M3.
   WingDrive already undoes rename, same-volume move and new folders
   (EXPL-003); FILE-008 extends that rather than replacing it.

## Acceptance Criteria

- [x] Items 1 and 2 ported, with a regression test each (#206, #210)
- [x] Items 3 and 4 ported or split into their own tasks
- [x] Items 5 to 7 classified again against current WingDrive code
- [x] `just ci-local` passes and the survey document is updated
