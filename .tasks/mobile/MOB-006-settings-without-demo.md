---
id: MOB-006
title: "Settings screen without demo content"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S03
tags: [mobile, interface]
last_updated: 2026-10-17
---

## Description

`src/screens/settings/SettingsScreen.tsx` shows sample buttons and switches. Profile, Security, Theme
picker and Clear cache only call `console.log`. Only reset works.

## Acceptance Criteria

- [x] Demo sections removed
- [x] Each remaining item works, wired to `config.app.update` where the desktop has the same setting
- [x] Strings in `en` and `pt`

## Evidence

- Sections: This Device (rename via `device.update`), General (language through `config.app.update`,
  auto-switch to synced libraries), Library (rename, thumbnails, sync, external volumes through
  `config.library.update`), Services, Privacy, Storage (`core.ephemeral_reset`, reset), About.
- The embedded core now skips `init_networking` when the networking switch is off.
- 81 translation keys used only by the demo removed. Verified on the Moto G7: language switch to
  Português applied at once and persisted (`config.app.update completed successfully`).
