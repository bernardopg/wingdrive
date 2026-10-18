---
id: MOB-008
title: "Add locations through the Android Storage Access Framework"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
tags: [mobile, android, core, locations]
last_updated: 2026-10-17
---

## Description

The app cannot add a location. On Android 11 and later the core needs a folder granted through the
Storage Access Framework or `MANAGE_EXTERNAL_STORAGE`. Decide the approach and wire `locations.add`.

## Acceptance Criteria

- [ ] Design note on SAF versus all-files access, including Play Store policy
- [ ] A user can pick a folder, index it and browse it
