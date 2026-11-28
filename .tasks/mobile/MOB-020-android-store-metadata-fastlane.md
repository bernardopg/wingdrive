---
id: MOB-020
title: "Android: store metadata and screenshots in the Fastlane layout"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: High
milestone: M5
sprint: S08
tags: [mobile, android, metadata, i18n, google-play, f-droid]
related_tasks: [DIST-002, DIST-006, DIST-007]
last_updated: 2026-11-28
---

## Description

F-Droid and IzzyOnDroid read the listing from `fastlane/metadata/android/<locale>/` in the
source repository, and Play can take the same files through the API. One copy in the repository
keeps the three listings equal.

## Implementation Steps

- [ ] Create `fastlane/metadata/android/en-US` and `pt-BR` with `title.txt` (30 chars), `short_description.txt` (80 chars), `full_description.txt` (4000 chars)
- [ ] Add `images/icon.png` (512x512), `images/featureGraphic.png` (1024x500) and 4 to 8 `images/phoneScreenshots`
- [ ] Add `changelogs/<versionCode>.txt` (500 chars) from the release workflow, using the CHANGELOG summary
- [ ] Add a CI check for the length limits

## Acceptance Criteria

- [ ] Both locales have every text file within the limits and at least 4 phone screenshots
- [ ] The tag run writes the changelog file for its versionCode
