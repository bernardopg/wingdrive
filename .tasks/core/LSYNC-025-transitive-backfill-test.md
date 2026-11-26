---
id: LSYNC-025
title: transitive_sync_backfill_test passes again
status: Done
assignee: bernardopg
parent: LSYNC-000
priority: Medium
milestone: M4
sprint: S06
tags: [core, sync, testing]
related_tasks: [LSYNC-024, NET-002]
last_updated: 2026-11-26
---

## Description

`core/tests/transitive_sync_backfill_test.rs` (Alice indexes, pairs with Bob, Bob pairs with Carol,
Carol is proxy-paired to Alice and syncs her data) failed before and after #551. CI does not run it,
so nobody noticed. Four separate problems, all in the test, found on 2026-11-26:

1. **Indexing timeout.** Alice content-indexed the whole repository on a cold debug build and hit
   the 120 s wait before pairing even started.
2. **Root entry without a volume.** The test creates the location without a volume and links only
   the location afterwards. The root entry keeps `volume_id = NULL`, and entries sync through their
   volume's owner, so the root is never sent. Bob's `sync.db` showed
   `Stuck dependencies: 209 records waiting for 9 missing parents` (the root, then 8 folders), with
   0 entries applied. Real locations are not affected: on the author's laptop every root entry has
   a volume.
3. **Peers left too early.** Alice and Bob slept a fixed 30-45 s after Bob paired with Carol and
   then exited, while Carol waits up to 120 s for Alice's data.
4. **Wrong voucher.** `auto_vouch_to_all` was set on Alice, but the device that pairs with Carol
   is Bob, so nobody vouched Carol to Alice and Carol only ever synced with Bob (0 of Alice's
   entries). `proxy_pairing_test` sets it on the device that does the new pairing.

## Implementation

- Alice indexes a generated tree of 8 folders with 25 files each, beside her data dir.
- After indexing, the root entry gets the location's volume, like the other sync tests do.
- Alice and Bob stay up until Carol reports success (at most 240 s).
- Bob, who pairs with Carol, has `auto_vouch_to_all`; Alice accepts vouches by default.

Side finding, not changed here: Bob advanced his entry watermark although all 209 entries were
stuck waiting for parents. Had the parents never been sendable, only a reconciliation (LSYNC-024)
would have fetched them again.

## Implementation Steps

- [x] Small generated fixture instead of the repository.
- [x] Give the root entry the location's volume.
- [x] Keep Alice and Bob up until Carol finishes.
- [x] Bob vouches Carol to Alice.
- [x] Test passes locally (2026-11-26: Carol received all 209 of Alice's entries, 412.6 s).

## Acceptance Criteria

- [x] `cargo test -p wing-core --test transitive_sync_backfill_test` passes.
