---
id: DEV-008
title: Clear the board-check warnings in .tasks/
status: To Do
assignee: bernardopg
parent: DEV-000
priority: Medium
milestone: M3
sprint: S05
tags: [dev, planning, tasks]
last_updated: 2026-11-16
---

## Description

The first `just board-check` (DEV-007) found 30 warnings: 23 tasks marked Done with unchecked items, `LSYNC-010` and `LSYNC-013` depending on `LSYNC-014` to `LSYNC-016`, which do not exist, and the S05 WIP at 6 of 2 because the INTG tasks wait on Google and Microsoft.

## Implementation Steps

- [ ] For each Done task with unchecked items: check the item with its evidence, or move what is left to a new task and drop the line.
- [ ] Fix or remove the `depends_on` entries of `LSYNC-010` and `LSYNC-013`.
- [ ] Decide how a task waiting on a third party is shown (back to To Do with a note, or a `blocked` tag the board surfaces), so the WIP counts real work.

## Acceptance Criteria

- [ ] `just board-check` reports 0 errors and 0 warnings on `main`.
