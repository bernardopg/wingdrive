---
id: TAURI-012
title: Runtime Regression Matrix on macOS and Windows
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M3
tags: [tauri, testing, ci, macos, windows]
last_updated: 2026-11-04
sprint: S05
---

## Description

Split from TAURI-006 so M1 can close on Linux. Repeat the TAURI-006 checklist on macOS and Windows and capture the result in CI where possible.

## Acceptance Criteria

- [ ] TAURI-006 checklist passes on macOS
- [ ] TAURI-006 checklist passes on Windows
- [ ] Terminal state captured on macOS and Windows CI runners

## Progress

2026-11-04: TAURI-016 makes the app build, test and open on Windows, so the Windows
round of this matrix can run. The TAURI-006 checklist itself has not run there yet.
