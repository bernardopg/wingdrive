---
id: MOB-016
title: "Android adaptive icon inside the safe zone, with a themed variant"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S03
tags: [mobile, android, branding]
last_updated: 2026-10-17
---

## Description

The launcher icon used the full-bleed 1024 px `icon.png` as the adaptive foreground, so every mask
cut the wing. There was no monochrome layer for Android 13 themed icons, and the background was black.

## Acceptance Criteria

- [x] Foreground and monochrome layers generated from `packages/assets/images/WingDriveLogo.svg` with
      the farthest point 31 dp from the centre (safe zone 33 dp); sources in `apps/mobile/assets/icon/`
- [x] Background `#0F0A1F`; legacy square and round icons for API 24 and 25 at every density
- [x] `app.json` `adaptiveIcon` points at the same sources, so a future prebuild keeps them
- [x] Checked on the Moto G7 launcher
