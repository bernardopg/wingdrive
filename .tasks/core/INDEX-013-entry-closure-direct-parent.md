---
id: INDEX-013
title: Entry closure misses direct-parent rows for batched entries
status: Done
assignee: bernardopg
parent: INDEX-000
priority: High
milestone: M2
sprint: S03
tags: [indexing, database, closure, bug]
last_updated: 2026-10-15
related_tasks: [DESK-014]
---

## Description

Found while testing hard-link aggregation for DESK-014. The indexer builds an
entry's `entry_closure` rows by copying its parent's rows, but each entry's
depth-0 self row was only written at the end of the batch. An entry created in
the same batch as its parent therefore had no row naming that parent (or any
in-batch ancestor) as an ancestor. Any descendant query through the closure
(aggregation scope, location counts, subtree moves) could miss entries.

## Implementation Steps

1. `DatabaseStorage::create_entry_in_conn` writes the direct-parent row
   explicitly instead of relying on the parent's pending self row.
2. Migration `m20261015_000001_repair_entry_closure` rebuilds every missing
   (ancestor, descendant) pair from `parent_id` chains in existing libraries,
   keeping current rows.
3. Regression tests: indexing a nested tree must give each entry closure
   ancestors equal to its `parent_id` chain; the migration restores the rows
   lost by a batched insert.

## Acceptance Criteria

- [x] New indexing writes complete closure rows for nested entries
- [x] Existing libraries are repaired by a migration
- [x] `test_closure_matches_parent_chain_for_nested_entries` fails without the
      fix and passes with it

## Notes

- `file_move_test` and `folder_rename_test` each have 2 failing ephemeral
  tests that fail identically on `main` before this change; not addressed here.
- DESK-014's hard-link aggregation walks `parent_id` in memory; it stays that
  way because it avoids a closure query per linked file.
