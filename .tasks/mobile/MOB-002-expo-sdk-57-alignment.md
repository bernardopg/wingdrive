---
id: MOB-002
title: "Align dependencies and native module with Expo SDK 57"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, dependencies, android]
last_updated: 2026-10-17
---

## Description

`expo-doctor` fails 6 checks. `modules/wing-mobile-core/package.json` depends on
`expo-modules-core ~3.0.28` (SDK 54), 11 packages are off the SDK 57 patch versions, `expo-router`
brings its own `react-native-screens`, `app.json` has the removed `newArchEnabled`, and the module's
Gradle file pins SDK 35 and `kotlin-stdlib-jdk7:1.9.25`.

## Acceptance Criteria

- [x] `wing-mobile-core` no longer installs its own `expo-modules-core`
- [x] `bunx expo install --check` reports no outdated package
- [x] `bunx expo-doctor` passes, or each remaining failure is documented as intentional
- [x] Module Gradle file uses the app's SDK levels and no explicit Kotlin stdlib
- [x] `bun run typecheck` passes and the app still launches

## Evidence

- `wing-mobile-core` imports from `expo` and declares `expo` as a peer; `expo-modules-core` removed
  from the app's direct dependencies; `newArchEnabled` and `unstable_enableSymlinks` removed.
- `bunx expo install --fix`: expo 57.0.30, expo-router 57.0.28 and 9 other patch alignments.
- Module `build.gradle` reads SDK levels from `rootProject.ext`; Kotlin stdlib line removed.
- `expo-doctor` 19/21. Intentional: the duplicate check flags Bun's isolated layout (same version
  linked twice) and `react-native-screens` 4.29 inside expo-router, which Metro now resolves to the
  app's 4.26.2; the non-CNG check flags the committed `android/` folder.
- expo-router 57.0.28 typed `Href` exposed that Recents, Favorites and File Kinds pushed routes that
  do not exist on mobile; those space items are hidden until MOB-007.
