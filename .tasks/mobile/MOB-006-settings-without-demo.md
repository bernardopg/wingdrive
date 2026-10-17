---
id: MOB-006
title: "Settings screen without demo content"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
tags: [mobile, interface]
last_updated: 2026-10-17
---

## Description

`src/screens/settings/SettingsScreen.tsx` shows sample buttons and switches. Profile, Security, Theme
picker and Clear cache only call `console.log`. Only reset works.

## Acceptance Criteria

- [ ] Demo sections removed
- [ ] Each remaining item works, wired to `config.app.update` where the desktop has the same setting
- [ ] Strings in `en` and `pt`
