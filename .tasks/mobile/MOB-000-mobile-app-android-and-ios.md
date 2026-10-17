---
id: MOB-000
title: "Mobile app: functional Android, fast dev loop and Tauri parity"
status: In Progress
assignee: bernardopg
priority: High
milestone: M5
tags: [epic, mobile, android, ios]
last_updated: 2026-10-17
---

## Description

Review on 2026-10-17 on the Moto G7 (Android 15) and the Android 16 emulator. The app crashes at
launch, its native module pulls an Expo SDK 54 runtime, and every change needs a release APK. The app
uses 13 core operations against about 75 in `packages/interface`. Pulled forward from M5 by the
author's decision on 2026-10-17.

## Children

- MOB-001 Launch crash: duplicate `@tanstack/react-query` in the bundle
- MOB-002 Align dependencies and native module with Expo SDK 57
- MOB-003 Fast dev loop: debug dev client, emulator, agent tooling
- MOB-004 Native core logs and JNI error handling
- MOB-005 Android manifest hardening: backup and unused permissions
- MOB-006 Settings screen without demo content
- MOB-007 File operations parity with the desktop
- MOB-008 Add locations through the Android Storage Access Framework
- MOB-009 Release signing and versioning for Android
- MOB-010 iOS dev client on macOS or EAS
- MOB-011 Android build gate in CI
- MOB-012 Multi-device pairing: real Network screen and device identity
- MOB-013 Refresh peer device info on reconnect and propagate unpairing
- MOB-014 Fix wing library sync-setup action names and instance device lookup

## Acceptance Criteria

- [ ] All children Done
- [ ] One day of real use on a phone paired with the desktop without a crash
