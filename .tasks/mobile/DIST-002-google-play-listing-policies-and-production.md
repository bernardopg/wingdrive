---
id: DIST-002
title: "Google Play: store listing, policy declarations and production release"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: High
milestone: M5
sprint: S10
tags: [android, google-play, policy, privacy]
related_tasks: [DIST-001, MOB-020, INTG-002]
last_updated: 2026-11-28
---

## Description

Everything Play reviews besides the binary. The All files access declaration is the main review
risk: the app requests `MANAGE_EXTERNAL_STORAGE` (`apps/mobile/app.json`), which Play allows for
file managers only after the Permissions Declaration Form and a demo video.

## Implementation Steps

- [ ] Store listing in en-US and pt-BR from the Fastlane metadata (MOB-020): title, short and full description, icon, feature graphic, phone and tablet screenshots
- [ ] Add an Android section to https://wingdrive.org/privacy: camera for pairing QR codes, local network discovery, All files access, no data sent to WingDrive servers
- [ ] Data safety form: no data collected by the developer; user-initiated transfers to Google Drive and OneDrive; encryption in transit
- [ ] All files access declaration with a demo video recorded on test files
- [ ] Camera permission justification (pairing QR code)
- [ ] Content rating (IARC), target audience 18+, no ads, app access notes for reviewers
- [ ] Check the app on a tablet and a foldable: with target SDK 36 Android 16 ignores `screenOrientation="portrait"` on large screens
- [ ] Promote the closed test release to production with a staged rollout

## Acceptance Criteria

- [ ] The All files access declaration is approved
- [ ] WingDrive is public on Google Play in Brazil and every other country
