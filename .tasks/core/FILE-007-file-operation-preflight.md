---
id: FILE-007
title: Preflight plan for copy, move and delete
status: Done
assignee: bernardopg
parent: FILE-000
priority: Medium
milestone: M3
tags: [files, operations, copy, delete, preview, upstream, port]
last_updated: 2026-11-04
related_tasks: [FILE-008, EXPL-003, DESK-018]
---

## Description

Split from item 7 of the 2026-10-14 upstream survey (upstream `3cd44f2da`). Before a copy, move or
delete runs, the daemon answers a read-only plan built from the index: how
many files and bytes, which destinations already exist and what the conflict
policy will do to each, and, for delete, which files hold the last copy of
their content in the library. The UI shows the plan in the confirmation
dialog and the CLI prints it before its prompt, with `--dry-run` to stop
there.

Today WingDrive validates through `ValidationResult` /
`RequiresConfirmation` in `core/src/infra/action/manager.rs`, and conflicts
are found by the job while it runs. Upstream replaced that hook with a
preview query that returns an `FsPlan`; port the idea, not the code, since
the action layer and the CLI have diverged.

## Implementation Notes

- One `files.preflight` query (or a preview per action) returning counts,
  bytes and per-path decisions; keep it read-only and bounded so a huge tree
  answers in time.
- Reuse the conflict policy the copy job already applies, so the plan and
  the job cannot disagree; a test runs both over one fixture.
- Delete's "last copy" count reads content identity from the index; paths
  outside any location report "unknown", not "safe".

## Acceptance Criteria

- [x] Copy, move and delete answer a preflight plan without touching the disk
- [x] The plan's conflict decisions match what the job does (shared fixture test)
- [x] Delete reports files that are the last copy of their content
- [x] The confirmation dialog and `wing file copy` and `wing file delete` with `--dry-run` show the plan
- [x] TypeScript types regenerated; docs in `docs/core/` updated

## Evidence

- `core/src/ops/files/plan.rs`: `final_destination`, `decide` (`PlanDecision`) and `next_free_name`, read-only.
  `FileCopyJob` now calls them instead of its own copies of the rules.
- `files.preflight` (`core/src/ops/files/query/preflight.rs`): totals, per-source decisions (first 1000),
  `truncated` after 200,000 walked entries, and for delete `last_copies` / `unknown_files` from content
  identity in the index (files outside the index or without content identity are unknown).
- Tests: `core/tests/preflight_test.rs` runs the plan and the copy job over one fixture for Skip and Keep both,
  checks `Ask` without a policy, and counts last copies for twins, a unique file, a folder and an unindexed
  file; unit tests for the plan rules and the bounded walk. `copy_action_test` (8) still passes.
- UI: the copy/move dialog shows files, size and what the chosen policy does to existing items; the delete
  dialog warns about last copies and unknown files. CLI: `wing file copy` and `wing file delete` print the
  plan before running, `--dry-run` stops there. Docs: `docs/core/file-copy-operations.mdx`.
- Desktop debug build, 2026-11-04 (isolated daemon, WebKitWebDriver): deleting `readme.txt` warned "1 file
  is the last copy of its content"; `twin-a.txt`, which has a twin, showed no warning. Copying
  `annual-report-2026.txt` onto a folder that had it showed "1 item already exists: 1 skipped", then
  "1 renamed" and "1 overwritten" as the policy changed; with Keep both the job wrote
  `annual-report-2026 (1).txt`. CLI `--dry-run` printed the same plan and copied nothing.
