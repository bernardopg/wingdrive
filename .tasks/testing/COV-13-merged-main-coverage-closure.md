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

## Evidence (2026-11-23, merged main same-SHA frontend)

- SHA: `9e0050e3494cfbfce3441795051b3f155c318c2f` (origin/main, includes PRs #508-#510).
- Frontend coverage: `bash scripts/coverage-frontend.sh /tmp/opencode/cov13-main/frontend` completed (58 shared test files + mobile suite, 0 fail).
- Report (`python3 scripts/coverage-report.py ... lcov.info`): Interface **73.7%** (5531/7532, 88 files) >= 60% floor. TS client 31.7% (591/1862), other packages 41.4% (2067/4996), mobile 100% (151/151). Frontend total 57.5%.
- Rust full coverage / 3x stability / ratchet same-SHA evidence: pending (heavy run; resource-constrained local environment).

## Blockers / Notes

- Full Rust coverage (`scripts/coverage-rust.sh`) is long-running and competes with other cargo jobs; 3 consecutive passes require controlled environment/time.
- Desktop 100% target remains aggressive; prior local full reports were ~90%. Any re-scope requires explicit approved task update.

