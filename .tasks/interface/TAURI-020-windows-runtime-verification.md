---
id: TAURI-020
title: Windows runtime acceptance verification (tray, logon, no console)
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S08
tags: [tauri, windows, verification, gap]
related_tasks: [TAURI-019, TAURI-017, DESK-003]
last_updated: 2026-11-22
---

## Description

TAURI-019 implemented no-console daemon startup, CREATE_NO_WINDOW spawn paths, and tray "Open Logs Folder". Real Windows machine verification is still pending because no Windows machine with the required environment is available in this session.

## Implementation Steps

- [ ] On a real Windows machine, verify login starts daemon and WingDrive with no console window.
- [ ] Verify closing all WingDrive windows leaves daemon running and tray icon present when "Keep running in background" is enabled.
- [ ] Verify tray Quit follows "Keep running" setting.
- [ ] Verify "Open Logs Folder" opens the correct logs directory on Windows.
- [ ] Verify "Start at login" behavior and scheduled task interaction.

## Acceptance Criteria

- [ ] All TAURI-019 acceptance criteria verified on Windows with evidence/screenshots/logs.
