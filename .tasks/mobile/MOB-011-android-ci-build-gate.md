---
id: MOB-011
title: "Android build gate in CI"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
tags: [mobile, android, ci]
last_updated: 2026-10-17
---

## Description

CI only typechecks the mobile app, so the Android build breaks without notice (jni 0.22, stale
Expo template and the launch crash all reached `main`). Add a job for the `mobile` CI scope.

## Acceptance Criteria

- [ ] `mobile` scope builds `wing-mobile-core` for one Android ABI and runs `gradlew assembleDebug`
- [ ] `bunx expo-doctor` runs in the same job
