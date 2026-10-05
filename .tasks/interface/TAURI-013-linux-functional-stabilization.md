---
id: TAURI-013
title: Linux desktop data-safety stabilization
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M1
sprint: S01
tags: [tauri, linux, safety, interface, regression]
last_updated: 2026-10-05
---

## Description

Fix the data-safety items from the desktop UI audit. File commands must act on what you see, and partial results must never look like success. Features that do nothing must stop pretending they work. Preserve existing data and contracts and avoid new runtime dependencies. Linux is the target.

The rest of the audit (startup recovery, auxiliary windows, search parity, clipboard/drag/pairing) moved to TAURI-014 so this slice fits S01.

## Implementation Steps

- [ ] Unify active collections, selection reconciliation, virtual-entity capabilities and operational directory
- [ ] Preserve partial copy results and represent cancelled, failed and skipped outcomes
- [ ] Remove unsupported encryption, sync, language and experimental promises; correct volume intersection
- [ ] Add targeted tests for selection, partial outcomes and hidden features

## Acceptance Criteria

- [ ] File commands operate on the current displayed selection, never virtual entities or stale paths
- [ ] Column operations target the active directory and exposed views honor their source
- [ ] Partial copy/move/duplicate outcomes remain visible and cannot be reported as integral success
- [ ] Visible preferences and redundancy assertions have real, testable behavior
- [ ] Experimental features are unavailable or truthful; Linux capabilities are exposed accurately
- [ ] Frontend tests, typecheck/build and relevant Rust validation pass, with limitations recorded

## Validation

Baseline: main b17f989a2; clean worktree; bun run typecheck passed.
