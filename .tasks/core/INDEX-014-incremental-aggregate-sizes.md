---
id: INDEX-014
title: Keep folder aggregate sizes current after incremental changes
status: Done
assignee: bernardopg
parent: INDEX-000
priority: Medium
milestone: M2
sprint: S03
tags: [indexing, explorer, sizes]
last_updated: 2026-10-28
---

## Description

`aggregate_size` of a directory is only computed in the aggregation phase of an indexing run
(`core/src/ops/indexing/phases/aggregation.rs`). Changes applied afterwards by the watcher or by
WingDrive's own file operations (create, delete, move, copy) never update it, so folder sizes in
the Explorer and the location totals in the Inspector go stale until the next rescan.

Found in the DESK-025 runtime check on 2026-10-27: after moving a 5 B file into a folder holding
2 B, the folder still showed 2 B, and the location still reported 3 files and 13 B. The opt-in
"Calculate Folder Sizes" preference (DESK-014) measures the disk instead and is not affected.

## Implementation Steps

1. When change detection creates, deletes or resizes an entry, apply the size delta to every
   ancestor directory through `entry_closure`.
2. A move subtracts the entry's share from the old ancestors and adds it to the new ones.
3. Location totals already follow watcher changes: `refresh_location_totals` recounts them with
   the library statistics, so they need no delta.
4. Integration test: index a folder, then create, grow, delete and move files and assert the
   aggregates after each change.

## Acceptance Criteria

- [x] Folder sizes match a fresh scan after create, delete, resize and move
- [x] Integration test covers each change kind

## Evidence

- `DatabaseAdapter` (the watcher's writer) applies each change's difference to the folders above
  it: `shift_entry_totals` on create, before a delete and on both sides of a move, and the size
  difference on update. `adjust_ancestor_totals` updates `aggregate_size` and `file_count` of every
  closure ancestor and `child_count` of the direct parent in single `UPDATE`s, bumps `indexed_at`
  so sync picks the totals up, and emits resource events for those folders so open listings show
  the new sizes. Failures are logged and never fail the change.
- Indexing runs are unaffected: their aggregation phase still recomputes every folder, so new
  directories (which the watcher hands to a background indexer job) end up exact.
- Known limit: a hard link created or removed through the watcher counts as a plain file until
  the next full scan, which removes the overcount again.
- `core/tests/aggregate_sizes_watcher_test.rs` drives create, resize, move and delete through the
  watcher and checks size, file count and child count after each; it failed before the change
  (`inner` stayed at 20 B after a 7 B file was created) and passes now. `file_move_test`,
  `folder_rename_test`, `entry_move_integrity_test` and `fs_watcher_test` still pass.
