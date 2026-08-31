---
id: TAURI-005
title: Implement External File Import
status: Done
assignee: unassigned
parent: TAURI-000
priority: Medium
tags: [tauri, explorer, drag-drop]
last_updated: 2026-08-26
---

## Description

Support dropping operating-system files into a physical Explorer destination. The previous handler prevented the native drop and discarded the files after logging their count.

## Acceptance Criteria

- [x] Reject drops without a writable physical destination
- [x] Validate conflicts before starting the operation
- [x] Dispatch the existing copy or ingestion job
- [x] Show progress and per-file failures
- [ ] Verify copy semantics on Linux, macOS, and Windows

## Implementation

- `Platform.onExternalFileDrop` (Tauri only) wraps the webview's native drag-drop events, so OS file drops reach the platform-agnostic interface.
- `useExternalFileDrop` in the Explorer subscribes once per window and resolves the visible folder through a ref so navigation never resubscribes mid-drag. Column view uses its deepest open folder rather than the root.
- `resolveExternalDrop` is a pure decision function with unit tests: drops outside a physical folder (virtual views, search, no destination) reject with an explicit toast instead of being swallowed.
- Valid drops open the file operation dialog, where conflicts are resolved before `files.copy` dispatches the copy job; progress and per-file failures flow through the Job Manager.
- The internal (app-level) drag system and OS drops are orthogonal: internal sessions emit `drag:*` app events and never trigger the native drop handler.

## Validation

- Typecheck, production Vite build, and the `resolveExternalDrop` bun tests pass.
- Live drop-to-import verification on Linux is pending the TAURI-006 runtime matrix; macOS and Windows require their CI runners.
