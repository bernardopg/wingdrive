---
id: TAURI-005
title: Implement External File Import
status: To Do
assignee: unassigned
parent: TAURI-000
priority: Medium
tags: [tauri, explorer, drag-drop]
last_updated: 2026-08-24
---

## Description

Support dropping operating-system files into a physical Explorer destination. The previous handler prevented the native drop and discarded the files after logging their count.

## Acceptance Criteria

- [ ] Reject drops without a writable physical destination
- [ ] Validate conflicts before starting the operation
- [ ] Dispatch the existing copy or ingestion job
- [ ] Show progress and per-file failures
- [ ] Verify copy semantics on Linux, macOS, and Windows
