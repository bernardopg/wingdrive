---
id: NAV-000
title: Multi-Window System
status: In Progress
assignee: jamiepine
parent: UI-000
priority: Medium
tags: [navigation, windows, architecture, upstream-backlog]
whitepaper: N/A
last_updated: 2026-10-16
milestone: M5
---

## Description

Current state: Settings, Inspector, Quick Preview, Job Manager, Spacebot, Spacedrop, context-menu, drag-overlay, floating-controls, and voice-overlay window routes exist and share library state. Independent Explorer windows, persistent geometry, and cross-window file drag/drop remain.

Implement support for multiple windows, each with independent navigation state. Users can open multiple Explorer windows, media viewers, or settings panels.

## Implementation Notes

- Each window has independent router state
- Tauri: use multi-window API
- Web: use browser windows or tabs
- Window state persists (size, position, route)
- Cross-window communication via events

## Acceptance Criteria

- [x] Open new Explorer window from menu (DESK-015, #217; runtime check in S04)
- [ ] Each window has independent navigation
- [ ] Window position/size persists
- [ ] Cross-window drag and drop
- [x] Windows share library state
- [ ] Close all windows command
- [ ] Platform-specific window controls (traffic lights on macOS)

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
