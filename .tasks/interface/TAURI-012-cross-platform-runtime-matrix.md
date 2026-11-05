---
id: TAURI-012
title: Runtime Regression Matrix on macOS and Windows
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M3
tags: [tauri, testing, ci, macos, windows]
last_updated: 2026-11-05
sprint: S05
---

## Description

Split from TAURI-006 so M1 can close on Linux. Repeat the TAURI-006 checklist on macOS and Windows and capture the result in CI where possible.

## Acceptance Criteria

- [ ] TAURI-006 checklist passes on macOS
- [ ] TAURI-006 checklist passes on Windows, on the installed `v3.0.0` (or later) `.exe` and `.msi`
- [ ] Terminal state captured on macOS and Windows CI runners

### Windows checks left over from TAURI-016

Each one is covered by unit tests and CI on the runner, but none has been done by hand on a real
Windows machine:

- [ ] Copy in WingDrive and paste in Explorer, and the other way round; cut and paste moves the files
- [ ] "Open with" and opening files with the default app (CORE-014)
- [ ] The daemon service registered through `schtasks` starts at login and stops cleanly
- [ ] A named instance (`WINGDRIVE_INSTANCE`) runs next to the everyday WingDrive
- [ ] A job that ends in the background shows a toast with the WingDrive logo
- [ ] SmartScreen and Smart App Control behavior on a fresh machine, recorded for REL-001

## Progress

2026-11-04: TAURI-016 makes the app build, test and open on Windows, so the Windows
round of this matrix can run. The TAURI-006 checklist itself has not run there yet.

2026-11-05: `v3.0.0` publishes the Windows installers, and `windows-bundle.yml` installs each one
on the runner, starts the daemon and the app, waits for the main window and uninstalls. That is
the part of "terminal state captured on CI" that exists for Windows; the checklist above still
needs a person at the machine.
