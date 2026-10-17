---
id: MOB-011
title: "Android build gate in CI"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S03
tags: [mobile, android, ci]
last_updated: 2026-10-17
---

## Description

CI only typechecks the mobile app, so the Android build breaks without notice (jni 0.22, stale
Expo template and the launch crash all reached `main`). Add a job for the `mobile` CI scope.

## Acceptance Criteria

- [x] `mobile.yml` runs clippy on `wing-mobile-core` for `aarch64-linux-android` (`-D warnings`) and
      assembles the release APK with Gradle
- [x] ESLint (zero warnings), i18n key parity and `expo install --check` gate the app. `expo-doctor`
      stays out: its duplicate and non-CNG checks flag the Bun layout and the committed `android/`.
- [x] actionlint and zizmor (medium and above) report nothing for `mobile.yml`
