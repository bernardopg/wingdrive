---
id: MOB-018
title: "Android: scan pairing QR codes without Google Play Services or ML Kit"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: High
milestone: M5
sprint: S09
tags: [mobile, android, f-droid, dependencies]
related_tasks: [DIST-006, DIST-007, MOB-019]
last_updated: 2026-11-28
---

## Description

`apps/mobile/src/components/PairingPanel.tsx` scans the pairing QR code with `CameraView` from
`expo-camera`. On Android, `expo-camera` 58 adds `com.google.android.gms:play-services-code-scanner`,
`com.google.mlkit:barcode-scanning` and `play-services-mlkit-document-scanner` unless the Gradle
properties `expo.camera.barcode-scanner-enabled` and `expo.camera.document-scanner-enabled` are
false. F-Droid rejects Play Services and IzzyOnDroid flags proprietary libraries, so the APK must
not contain them. The document scanner is never used.

## Implementation Steps

- [ ] Set `expo.camera.document-scanner-enabled=false` in `apps/mobile/android/gradle.properties` (and the Expo config plugin, so prebuild keeps it)
- [ ] Replace the ML Kit barcode path with a FLOSS decoder (zxing-cpp or ZXing on CameraX frames) in all builds, then set `expo.camera.barcode-scanner-enabled=false`
- [ ] Check that the Gradle dependency tree of `:app:releaseRuntimeClasspath` has no `com.google.android.gms`, `com.google.mlkit` or `com.google.firebase`
- [ ] Add that check to the Android CI job so the dependencies do not come back

## Acceptance Criteria

- [ ] Pairing by QR code works on a phone without Google Play Services (emulator image without Google APIs)
- [ ] `./gradlew :app:dependencies` lists no Play Services, ML Kit or Firebase artifact
