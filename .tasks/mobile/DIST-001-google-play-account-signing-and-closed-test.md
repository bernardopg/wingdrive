---
id: DIST-001
title: "Google Play: developer account, app signing key and the 14-day closed test"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: High
milestone: M5
sprint: S08
tags: [android, google-play, release, signing]
related_tasks: [MOB-009, REL-001, MOB-020]
last_updated: 2026-11-28
---

## Description

New personal Play Console accounts must run a closed test with at least 12 testers opted in for
14 consecutive days before they can apply for production. This is the longest lead time of every
store, so it starts first. The signing decision is permanent: the APKs on GitHub Releases are
already signed with the key in `WINGDRIVE_ANDROID_KEYSTORE_*`, and Play must use that same key as
its app signing key, or users cannot move between Play and GitHub builds without uninstalling.
See "Google Play" in `docs/design/app-store-distribution.md`.

## Implementation Steps

- [ ] Create a personal Play Console account (USD 25 once), verify identity and the Android device
- [ ] Create the app `com.wingdrive.app`, default language en-US, free, category Productivity
- [ ] Play App Signing: choose "Use a different key" and upload the current release key with the PEPK tool
- [ ] Generate a new upload key, store it in Bitwarden and register it in Play Console
- [ ] Upload a signed AAB from a tag run to the internal track, then to a closed track
- [ ] Recruit at least 20 testers (Discord, friends) and keep 12 or more opted in for 14 days
- [ ] Apply for production and answer the closed test questionnaire

## Acceptance Criteria

- [ ] The GitHub Release APK and the Play build have the same signing certificate (`apksigner verify --print-certs`)
- [ ] The closed test ran 14 days with 12 or more testers and production access was granted
