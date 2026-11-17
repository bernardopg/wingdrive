---
id: MOB-009
title: "Release signing and versioning for Android"
status: In Progress
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S10
tags: [mobile, android, release]
last_updated: 2026-11-17
---

## Description

Release builds are signed with the debug keystore, and the app stays at `1.0.0` / `versionCode 1`
while the desktop is `2.0.0-alpha.18`. Coordinates with REL-001 (Google Play credentials).

## Acceptance Criteria

- [x] Release keystore outside the repository, read from `WINGDRIVE_ANDROID_KEYSTORE*` env (CI decodes it from secrets)
- [x] `versionName` follows the tag; `versionCode` from `scripts/release/android_version.py`, self-tested
- [ ] Signed AAB builds locally

## Evidence

- `release.yml` calls `mobile.yml` with the tag; the APK, AAB and `SHA256SUMS-android` join the GitHub Release when the upload key secrets exist.
- Pending: generate the upload key and add the four secrets (with REL-001), then one signed tag run.

## Progress (2026-11-17)

#418: `apps/mobile/app.json` and `apps/mobile/package.json` carry the product version, `cargo xtask
bump` updates them and `scripts/release/version.py` checks them against the tag. Builds without the
release properties take `versionName` from `app.json` and derive `versionCode` with the scheme of
`android_version.py`, instead of 1.0.0 and 1.
