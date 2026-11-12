---
id: INTG-004
title: End-to-end validation with real Google on every platform
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S06
tags: [integrations, google, testing, linux, windows, macos]
last_updated: 2026-11-12
related_tasks: [CLOUD-004, TAURI-012]
---

## Description

CLOUD-004 was verified against a fake Google (`core/tests/google_integration_test.rs`) and an isolated `wing-server`. It has never talked to Google itself or run in a packaged app.

## Implementation Steps

1. With the INTG-001 client, sign in from the Linux packages (deb, AUR, AppImage), Windows (`.exe`) and the `wing-server` web UI; macOS when TAURI-012 has a machine.
2. Check the loopback redirect through the desktop firewall on Windows and inside the Flatpak sandbox.
3. Run the matrix: two accounts, Drive browse with duplicates and Google Docs, upload and download over 1 GB with a dropped connection, rename and move, trash, local editing with a conflict, two-way sync with deletions, Photos upload to an album, picker import, disconnect.
4. Revoke access from https://myaccount.google.com/permissions and confirm WingDrive asks to reconnect (INTG-005).
5. Record results in the task log and file bugs as DESK/INTG tasks.

## Acceptance Criteria

- [ ] Every step of the matrix passes on Linux and Windows packages.
- [ ] Results recorded with versions and dates.
