---
id: DESK-014
title: "Show accurate directory content sizes without blocking navigation"
status: Done
assignee: "bernardopg"
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, core, explorer, performance]
last_updated: 2026-10-15
---

## Description

Explorer currently presents Linux directory metadata bytes as content size for unindexed folders. Show only verified totals, preserve immediate folder navigation, and let people opt into a bounded background calculation for the current directory.

## Implementation Steps

- [x] Expose a separate optional directory-content total in the file API.
- [x] Return indexed aggregate sizes and omit unindexed totals.
- [x] Add a serialized, on-demand directory-size query for physical paths.
- [x] Add a localized Explorer preference to request background size calculation.
- [x] Cover the aggregation and UI-size selection behavior with focused tests.

## Acceptance Criteria

- [x] An unindexed folder never displays its filesystem metadata length as content size.
- [x] Indexed folders display their stored recursive aggregate, including `0 B` for empty folders.
- [x] Enabling folder-size calculation leaves directory listing responsive and scans at most one subtree at a time.
- [x] Inaccessible folders are reported as unavailable rather than assigned a partial total.
- [x] Core tests, generated TypeScript, typecheck, and scoped CI checks pass.

## Evidence

2026-10-15: `cargo test -p wing-core directory_sizes --lib`, `bun test packages/ts-client/src/fileUtils.test.ts`, `bun run typecheck`, and `just ci-local` passed. The local CI completed Rust, desktop, frontend, generated-type, formatting, and lint gates.
