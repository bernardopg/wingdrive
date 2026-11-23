---
id: COV-13
title: "S08 carryover: coverage closure on merged-main same-SHA evidence"
status: To Do
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
