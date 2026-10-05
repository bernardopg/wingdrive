---
id: TAURI-013
title: Linux desktop functional stabilization
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M1
tags: [tauri, linux, safety, interface, regression]
last_updated: 2026-10-04
---

## Description

Implement the first five stabilization priorities from the desktop UI audit. Preserve existing data and contracts, avoid external runtime dependencies, and make unsupported features explicitly unavailable rather than pretending they work. Linux is the target; cross-platform runtime certification and release publication are not part of this task.

## Implementation Steps

- [ ] Unify active collections, selection reconciliation, virtual-entity capabilities and operational directory
- [ ] Preserve partial copy results and represent cancelled, failed and skipped outcomes
- [ ] Remove unsupported encryption, sync, language and experimental promises; correct volume intersection
- [ ] Bound startup failure and recover isolated per-window event subscriptions
- [ ] Compose feature providers, dialogs, notifications and reconnect handling in auxiliary windows
- [ ] Complete search scope/filter/sort/pagination/debounce and view parity
- [ ] Preserve streamed text and improve clipboard, drag, Jobs, source retry and pairing
- [ ] Add targeted tests and Linux validation; review integration

## Acceptance Criteria

- [ ] File commands operate on the current displayed selection, never virtual entities or stale paths
- [ ] Column operations target the active directory and exposed views honor their source
- [ ] Partial copy/move/duplicate outcomes remain visible and cannot be reported as integral success
- [ ] Visible preferences and redundancy assertions have real, testable behavior
- [ ] Startup failures are recoverable; restart and window cleanup preserve scoped subscriptions
- [ ] Jobs and Settings auxiliary windows have their required contexts and feedback
- [ ] Search and preview regressions are covered by tests; pagination and source retries are explicit
- [ ] Experimental features are unavailable or truthful; Linux capabilities are exposed accurately
- [ ] Frontend tests, typecheck/build and relevant Rust validation pass, with limitations recorded

## Validation

Baseline: main b17f989a2; clean worktree; bun run typecheck passed. No commits or publication requested.
