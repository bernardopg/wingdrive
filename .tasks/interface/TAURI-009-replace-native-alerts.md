---
id: TAURI-009
title: Replace Native alert() Calls with Toasts
status: Done
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

- [x] `Settings/pages/GeneralSettings.tsx` reset success and error use toasts
- [x] `Settings/pages/ServicesSettings.tsx` restart notice uses a toast
- [x] `windows/FloatingControls.tsx` placeholder `alert("Stop!")` removed (demo window, no real action)
- [x] No `alert(` or `window.confirm` remains in `packages/interface/src`

## Safety Fix

"Reset All Data" used `window.confirm`, which returns `true` on WebView2 (Windows) without showing a dialog, so a single click would have wiped all libraries. It now uses `platform.confirm`, which the Tauri platform implements with a real dialog.

Verified with `bun run typecheck`.
