---
id: MOB-000
title: "Mobile app: functional Android, fast dev loop and Tauri parity"
status: In Progress
assignee: bernardopg
priority: High
milestone: M5
sprint: S09
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
- MOB-015 Android visual fixes: header buttons and system bar overlap
- MOB-016 Android adaptive icon inside the safe zone, with a themed variant
- MOB-017 Android review on the moto g(7): storage, empty views, translations, Wings

## Road to production (2026-11-16)

Order decided after the MOB-017 review. Items 1 to 4 make the app useful on one phone, 5 to 7 make
it shippable on Google Play, 8 to 10 make it a daily file manager.

1. **MOB-017 rest:** check on the moto g(7) that "All files access" shows the internal storage and
   cards, then find the 1 to 3 s tap delay (JS thread profile with Hermes, not frames: `gfxinfo`
   showed no slow frame).
2. **MOB-008 locations:** add a folder of the phone as a location (All files access first, SAF only
   for cards Android hides) so its files are indexed, searchable and synced. Write the Play Store
   note: `MANAGE_EXTERNAL_STORAGE` is allowed for file managers but needs the declaration form.
3. **MOB-007 file operations:** rename, delete to trash, new folder, copy and move, favorites and
   tags, with listings that refresh after each action.
4. **Open and share files:** open a file with the Android app that handles it (`ACTION_VIEW` through
   a `FileProvider`) and share it (`ACTION_SEND`); receive files shared to WingDrive.
5. **MOB-009 release signing:** upload keystore, the four `WINGDRIVE_ANDROID_KEYSTORE_*` secrets,
   version from the tag (`versionCode` derived from it), signed AAB in `release.yml`.
6. **MOB-005 manifest:** finish the hardening; check that QR pairing still opens the camera.
7. **Play Store listing:** internal testing track first, privacy policy (the site's `/privacy`
   already covers the data), data safety form, screenshots in EN and PT.
8. **Background sync:** done on Android by MOB-021, a foreground service the user turns on in
   Settings. Starting at boot is still open.
9. **Crash and ANR reporting** the user can opt into, since Android is not in daily use by the
   author.
10. **MOB-010 iOS:** dev client on a Mac or EAS once Android ships.

## Acceptance Criteria

- [ ] All children Done
- [ ] One day of real use on a phone paired with the desktop without a crash
