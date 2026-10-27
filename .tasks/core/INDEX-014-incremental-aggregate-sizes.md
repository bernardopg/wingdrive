---
id: INDEX-014
title: Keep folder aggregate sizes current after incremental changes
status: To Do
assignee: bernardopg
parent: INDEX-000
priority: Medium
milestone: M2
tags: [indexing, explorer, sizes]
last_updated: 2026-10-27
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
   ancestor directory through `entry_closure`, in the same transaction.
2. A move subtracts the size from the old ancestors and adds it to the new ones.
3. Keep the location totals (file count, size) in step with the same deltas.
4. Integration test: index a folder, then create, grow, delete and move files and assert the
   aggregates match a fresh scan.

## Acceptance Criteria

- [ ] Folder sizes and location totals match a fresh scan after create, delete, resize and move
- [ ] Integration test covers each change kind
