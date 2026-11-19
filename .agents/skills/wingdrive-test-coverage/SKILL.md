---
name: wingdrive-test-coverage
description: Measure and increase WingDrive test coverage for any component. Use for coverage targets, coverage regressions, or defects and diagnostics found while adding tests.
---

# WingDrive test coverage

Read [Coverage](../../../COVERAGE.md), the [coverage plan](../../../.tasks/testing/README.md), and the task for the component. Use [the project planning rules](../../../.tasks/PLANNING.md) for status and evidence.

1. Establish a baseline at a known SHA. Record platform, toolchain, runners, filters and covered/instrumented line counts. Prefer an existing full report on the same SHA or run the official collectors. Label older reports with their SHA. Compare exact counts. Desktop reaches 100% only when covered equals instrumented and any uninstrumented platform or entry points are disclosed.
2. Rank uncovered files and branches with the LCOV report. Choose behavior that matters at a normal entry point. Assert return values, filesystem or database state, exit status, progress, and user-visible errors as applicable. Use disposable fixtures, isolated processes and synthetic credentials. Register new suites in the normal CI and coverage runners. Keep platform boundaries explicit.
3. Capture command, exit status, test counts and full stdout/stderr under ignored `coverage/`. Inspect every warning, error, unexpected log and failure. Record each distinct diagnostic, its count and disposition in the task or PR. Protect credentials and personal paths before copying excerpts. Classify expected error-path logs, tool failures and confirmed product defects using evidence.
4. When a test exposes a defect, reproduce it with the smallest deterministic case. Record affected path and symbol, environment, expected and actual behavior, impact and confirmed cause in [BUGS.md](../../../.tasks/testing/BUGS.md), the owning task and Headroom memory. Fix the cause and keep the regression test. Verify the same reproducer, adjacent cases and the focused suite. Update the three records with the correction, test results and remaining limits.
5. Publish a reviewable PR for each completed batch when authorized. Include raw counts, measurement scope, diagnostic disposition and the linked task or issue. Follow the repository's [PR validation instructions](../../../AGENTS.md#pull-request-validation) and the user's monitoring preference. Use the next uncovered behavior while independent work can proceed.
6. Repeat until one comparable full report on the final tested commit proves the requested component targets. Report partial measurements as partial. Cite the measured SHA, LCOV and remaining gap for every unmet target.
