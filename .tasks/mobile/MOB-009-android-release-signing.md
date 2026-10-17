---
id: MOB-009
title: "Release signing and versioning for Android"
status: In Progress
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S03
tags: [mobile, android, release]
last_updated: 2026-10-17
---

## Description

Release builds are signed with the debug keystore, and the app stays at `1.0.0` / `versionCode 1`
while the desktop is `2.0.0-alpha.18`. Coordinates with FORK-002 (Google Play credentials).

## Acceptance Criteria

- [x] Release keystore outside the repository, read from `WINGDRIVE_ANDROID_KEYSTORE*` env (CI decodes it from secrets)
- [x] `versionName` follows the tag; `versionCode` from `scripts/release/android_version.py`, self-tested
- [ ] Signed AAB builds locally

## Evidence

- `release.yml` calls `mobile.yml` with the tag; the APK, AAB and `SHA256SUMS-android` join the GitHub Release when the upload key secrets exist.
- Pending: generate the upload key and add the four secrets (with FORK-002), then one signed tag run.
