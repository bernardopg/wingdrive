---
id: DESK-010
title: Startup latency and resident memory budget
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, performance]
last_updated: 2026-10-07
---

## Description

With the app resident, opening a folder must feel as fast as Dolphin. Cold start and memory are measured and kept within a budget.

## Implementation Steps

- [ ] Measure time from `wingdrive <dir>` to a painted listing, cold and resident
- [x] Daemon readiness polling starts at 25 ms instead of 100 ms; avoid waiting when the socket is already up
- [ ] Budgets: resident open < 300 ms, cold start < 2.5 s, idle PSS of app + daemon recorded in the smoke test

## Acceptance Criteria

- [ ] Measured numbers recorded in Evidence and enforced in the release smoke test

## Evidence

- Readiness poll is 25 ms instead of 100 ms and the app logs `elapsed_ms` when the daemon answers.
- `scripts/release/smoke.py` now asserts cold start to daemon ready under `WINGDRIVE_STARTUP_BUDGET_SECONDS` (20 s on CI runners) next to the existing PSS budget; the release run records both numbers.
