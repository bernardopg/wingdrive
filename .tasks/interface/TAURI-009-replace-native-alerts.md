---
id: TAURI-009
title: Replace Native alert() Calls with Toasts
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M1
sprint: S01
tags: [tauri, ux]
last_updated: 2026-10-01
---

## Description

Native `alert()` blocks the webview and looks foreign in the desktop app. Remaining calls are in settings pages and a placeholder floating control.

## Acceptance Criteria

- [ ] `Settings/pages/GeneralSettings.tsx` reset success and error use toasts
- [ ] `Settings/pages/ServicesSettings.tsx` restart notice uses a toast
- [ ] `windows/FloatingControls.tsx` placeholder `alert("Stop!")` is wired to a real action or removed
- [ ] No `alert(` remains in `packages/interface/src`
