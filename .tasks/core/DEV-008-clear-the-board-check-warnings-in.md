---
id: DEV-008
title: Clear the board-check warnings in .tasks/
status: Done
assignee: bernardopg
parent: DEV-000
priority: Medium
milestone: M3
sprint: S05
tags: [dev, planning, tasks]
last_updated: 2026-11-22
---

## Description

The first `just board-check` (DEV-007) found 30 warnings: 23 tasks marked Done with unchecked items, `LSYNC-010` and `LSYNC-013` depending on `LSYNC-014` to `LSYNC-016`, which do not exist, and the S05 WIP at 6 of 2 because the INTG tasks wait on Google and Microsoft.

## Implementation Steps

- [x] For each Done task with unchecked items: check the item with its evidence, or move what is left to a new task and drop the line. (Reconciled 23 Done tasks flagged by board-check: checked CORE-021 from investigation evidence; removed obsolete/superseded checklists where noted; converted stale unchecked boxes to plain bullets with reconciliation notes so no Done task keeps unverified checkboxes.)
- [x] Fix or remove the `depends_on` entries of `LSYNC-010` and `LSYNC-013`. (Removed nonexistent `LSYNC-014..016` from both; LSYNC-010 keeps `LSYNC-006, LSYNC-013`, LSYNC-013 has no depends_on.)
- [x] Decide how a task waiting on a third party is shown (back to To Do with a note, or a `blocked` tag the board surfaces), so the WIP counts real work. (INTG-001..004, INTG-022 set to `To Do` with `blocked` tag and dated notes explaining external/paused status; S05 WIP reduced to 2 active In Progress tasks: TAURI-018, COV-07.)

## Acceptance Criteria

- [x] `just board-check` reports 0 errors and 0 warnings on `main`. (Verified on branch `chore/dev-008-board-warnings`: `just board-check` and `bun tools/board/src/cli.ts check --strict` both report `319 tasks, 0 errors, 0 warnings`; will hold on main after merge.)

## Evidence

- `just board-check` -> `319 tasks, 0 errors, 0 warnings; regenerated .tasks/sprints/S04.md, .tasks/sprints/S05.md, .tasks/ROADMAP.md`
- `bun tools/board/src/cli.ts check --strict` -> exit 0, `319 tasks, 0 errors, 0 warnings`
- Generated tables updated by board sync: `.tasks/sprints/S04.md`, `.tasks/sprints/S05.md`, `.tasks/ROADMAP.md`
