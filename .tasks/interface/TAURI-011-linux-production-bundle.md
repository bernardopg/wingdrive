---
id: TAURI-011
title: Verify Linux Production Bundle
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
sprint: S01
milestone: M1
tags: [tauri, packaging, linux]
last_updated: 2026-10-02
---

## Description

Verify production Linux bundles with isolated daemon startup. The user explicitly deferred TAURI-006; its broader regression matrix remains outside this task.

## Acceptance Criteria

- [ ] `bun run tauri build` produces AppImage and deb on Linux
- [ ] Installed bundle starts the packaged daemon and connects the main window
- [ ] Packaged startup smoke check passes against the production bundle
- [ ] Wayland startup works or the GDK_BACKEND=x11 fallback is applied automatically
