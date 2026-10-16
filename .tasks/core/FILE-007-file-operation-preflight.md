---
id: FILE-007
title: Preflight plan for copy, move and delete
status: To Do
assignee: bernardopg
parent: FILE-000
priority: Medium
milestone: M3
tags: [files, operations, copy, delete, preview, upstream, port]
last_updated: 2026-10-16
related_tasks: [FORK-006, FILE-008, EXPL-003, DESK-018]
---

## Description

Split from FORK-006 item 7 (upstream `3cd44f2da`). Before a copy, move or
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

- [ ] Copy, move and delete answer a preflight plan without touching the disk
- [ ] The plan's conflict decisions match what the job does (shared fixture test)
- [ ] Delete reports files that are the last copy of their content
- [ ] The confirmation dialog and `wing file copy` and `wing file delete` with `--dry-run` show the plan
- [ ] TypeScript types regenerated; docs in `docs/core/` updated
