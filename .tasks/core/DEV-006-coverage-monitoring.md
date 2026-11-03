---
id: DEV-006
title: Coverage for every component, published without manual edits
status: Done
assignee: bernardopg
parent: DEV-000
priority: Medium
tags: [ci, testing, coverage, documentation]
last_updated: 2026-11-03
---

## Description

The README needed one coverage badge, and a page that shows coverage per component (core,
desktop, CLI, server, mobile, crates, interface, TypeScript client) that follows the workflows
without anyone editing numbers.

## Acceptance Criteria

- [x] Rust coverage of the tests CI runs, plus the CLI, server and crate unit tests (`cargo-llvm-cov`)
- [x] Frontend coverage of the `bun test` files CI runs
- [x] One merged report, per-component numbers, badges for the README and COVERAGE.md
- [x] HTML report published on GitHub Pages; summary on the workflow run page
- [x] Runs on every push to `main` and weekly, with no external account or secret

## Evidence

- `scripts/coverage-rust.sh`, `scripts/coverage-frontend.sh` and `scripts/coverage-report.py`;
  `.github/workflows/coverage.yml` publishes the badges on the `badges` branch and the report on
  https://bernardopg.github.io/wingdrive/.
- Local run, 2026-11-03: total 24.0% (26,532 of 110,755 lines), Rust 22.0%, frontend 40.8%;
  core 19.5%, desktop 46.1%, shared crates 48.3%, interface 43.8%, TypeScript client 31.2%,
  CLI 0.4%, server 0.8%. The mobile core has no tests.
- `scripts/ci-rust.sh tests` runs only the test half of the full gate, so coverage measures the
  same tests as CI.
