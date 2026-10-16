---
id: DESK-013
title: Finish i18n and theming for the Linux desktop
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [i18n, themes, desktop, linux]
last_updated: 2026-10-16
---

## Description

PR #201 translated the interface to Brazilian Portuguese (i18next, Crowdin),
added 12 themes, light/dark icons and a redesigned Settings, and moved Linux to
the in-app context menu. Native and unverified parts remain.

## Implementation Steps

1. Translate the tray menu (`apps/tauri/src-tauri/src/background.rs`: "Open
   WingDrive", "New Tab", "Quit WingDrive") from `preferences.language`, and
   rebuild it when `ConfigChanged { field: "language" }` arrives.
2. Open external links on the About page with `platform.openLink` instead of
   `<a target="_blank">`, which stays inside the webview in Tauri.
3. Proofread the Portuguese strings in Crowdin and fix wording that reads as a
   literal translation.
4. Verify the packaged AppImage on Hyprland: themes, light icons, context menu
   position and no menu bar.
5. Verify native context menu position on macOS and Windows (cursor position is
   now passed to `menu.popup`).

## Acceptance Criteria

- [x] Tray menu follows the UI language
- [x] About links open the system browser from the desktop app
- [ ] pt-BR proofread to 100% in Crowdin
- [ ] AppImage verified on Linux; macOS/Windows menu position checked or
      recorded as blocked on hardware
