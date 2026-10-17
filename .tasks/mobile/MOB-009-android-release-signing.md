---
id: MOB-009
title: "Release signing and versioning for Android"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
tags: [mobile, android, release]
last_updated: 2026-10-17
---

## Description

Release builds are signed with the debug keystore, and the app stays at `1.0.0` / `versionCode 1`
while the desktop is `2.0.0-alpha.18`. Coordinates with FORK-002 (Google Play credentials).

## Acceptance Criteria

- [ ] Release keystore outside the repository, read from Gradle properties or env
- [ ] `versionName` follows the desktop version; `versionCode` increases per release
- [ ] Signed AAB builds locally
