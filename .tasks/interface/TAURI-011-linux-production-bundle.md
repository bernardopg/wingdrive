---
id: TAURI-011
title: Verify Linux Production Bundle
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
sprint: S02
milestone: M1
tags: [tauri, packaging, linux]
last_updated: 2026-10-01
---

## Description

Only the dev executable has been validated. M1 requires a production bundle that starts its own daemon and passes the TAURI-006 checks.

## Acceptance Criteria

- [ ] `bun run tauri build` produces AppImage and deb on Linux
- [ ] Installed bundle starts the packaged daemon and connects the main window
- [ ] TAURI-006 Linux checklist passes against the installed bundle
- [ ] Wayland startup works or the GDK_BACKEND=x11 fallback is applied automatically
