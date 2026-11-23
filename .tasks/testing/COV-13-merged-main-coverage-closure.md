---
id: COV-13
title: "S08 carryover: coverage closure on merged-main same-SHA evidence"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S08
tags: [coverage, testing, ci, carryover]
related_tasks: [COV-00, COV-11, COV-12]
last_updated: 2026-11-23
---

## Description

COV PRs merged with targeted tests, but full coverage targets remain unproven on merged-main same-SHA evidence. COV-12 audit recorded Core 55.4% (<65%), Desktop 90.7% (<100%), CLI met 75% floor, Interface needs same-SHA confirmation. Local full runs were limited by SIGTERM/resource pressure.

## Acceptance Criteria

- [ ] Run full coverage collectors on merged main same SHA and publish artifacts.
- [ ] Core >=65%, Desktop >=100% (or explicitly re-scoped with approved task update), CLI >=75%, Interface >=60% evidenced.
- [ ] Run 3 consecutive full-suite passes on same env/SHA with no flakiness/state leakage.
- [ ] COV-11 ratchet enforced on PR scope without duplicating heavy builds.

## Evidence (2026-11-23)

- Frontend same-SHA (`9e0050e34`): `scripts/coverage-frontend.sh` + `coverage-report.py` -> Interface **73.7%** (5531/7532) >= 60%; TS client 31.7%; other packages 41.4%; mobile 100%; frontend total 57.5%; 0 failures.
- TS client coverage boosted and merged in PR #513: added tests for `deviceIcons`, `volumeIcons`, `virtualFiles`, `fileUtils`, `sidebar` store, `sortPreferences`.
- GHA Coverage workflow on merged main: run 45121508674 (push from #512 merge) has Frontend tests success; Rust tests still in_progress at time of update. Full Rust coverage is delegated to GHA per user direction.
- 3 consecutive full-suite passes: not run locally per user direction ("não precisa rodar 3x; full rust já roda no GHA").

## Blockers / Notes

- Core >=65%, Desktop >=100%, CLI >=75% same-SHA full evidence depends on completed GHA coverage run artifacts/report.
- Desktop 100% remains an aggressive target (prior local ~90.2%); re-scope requires explicit approved task update if kept.
- COV-11 ratchet: `scripts/coverage-ratchet.py` exists in CI workflow; PR-scope enforcement validated by CI, not duplicated locally.

