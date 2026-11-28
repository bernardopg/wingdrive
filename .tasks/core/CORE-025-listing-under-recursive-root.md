---
id: CORE-025
title: Folders below a recursively indexed root list as empty when they are new or missing
status: In Progress
assignee: bernardopg
parent: CORE-000
priority: High
milestone: M3
sprint: S06
tags: [core, indexing, explorer, bug, flaky]
related_tasks: [CORE-021, TAURI-018]
last_updated: 2026-11-28
---

## Description

The Explorer smoke kept failing now and then after CORE-021 closed. In the CI of #551 the step that
opens a deleted location showed "0 files" instead of the alert "Could not open this location", and
the `v3.2.0` release smoke opened a folder with 10,002 items as "0 files" (CORE-021 never explained
that one).

Cause: the ephemeral index cache counts a path as indexed when a `Recursive` root covers it, such as
the volume walk of `/tmp`. The listing then served the path from the index, and a path with no node
there lists as empty. Two kinds of paths have no node under a finished walk:

- a folder created after the walk (the smoke's fixture, created right after the server starts);
- a folder that no longer exists (the smoke's `deleted-location`).

Whether the walk finished before the smoke opened the folder decided the result, so the failure was
intermittent.

## Implementation

`query_ephemeral_directory_impl` serves from the cache only when the folder has a node in the index,
or when it was scanned itself (an empty folder has no node) and is still a directory on disk.
Anything else falls through to the disk: a missing folder fails with the I/O error the Explorer
shows, and a new one gets its own shallow scan. `EphemeralIndexCache::is_indexed_directly` tells a
scanned folder from a covered one.

## Implementation Steps

- [x] Fall through to the disk for covered paths without a node.
- [x] Regression test `paths_under_recursive_root_without_node_read_the_disk` in
      `directory_listing_first_response_test`: a folder created after the walk lists its files and a
      missing one fails.
- [x] Explorer smoke passes in CI on the pull request (#554, 19m53s). The 3.8.2 release smoke on
      the packaged bundles passed too. The scheduled CI run of 2026-11-26 failed on `ce7f4ec`, before
      #554 merged; the next scheduled runs decide the last criterion.
- [x] Second cause, in the interface. The scheduled run of 2026-11-27 (46020831007, run 6 of 10, on
      `9398edbde`, after #554) failed at the same step: the alert appeared, was detached, and the
      folder showed "0 files". The daemon answers the missing path with an error every time.
      TanStack Query clears `error` and goes back to `pending` when a query with no data fetches
      again, and a focus or change event refetched the listing, so the alert was gone for about
      1.5 s per refetch (one retry with a 1 s delay). `useStickyError` keeps the last error of the
      active Explorer query until a fetch succeeds or the path changes. The smoke now sends a
      `visibilitychange` with the alert shown and requires it to stay visible for 1.5 s. Local
      check on 2026-11-28 with an isolated `wing-server`: the smoke fails on that step without the
      fix ("The missing location error disappeared during a refetch") and passes four runs in a row
      with it.

## Acceptance Criteria

- [x] A folder created under a walked root lists its entries on the first open.
- [x] A missing folder under a walked root shows the error, not an empty folder.
- [ ] The scheduled smoke runs of TAURI-018 stay green.
