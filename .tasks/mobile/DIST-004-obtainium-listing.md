---
id: DIST-004
title: "Obtainium: document installing and updating the Android app from GitHub Releases"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Low
milestone: M5
sprint: S08
tags: [android, distribution, docs, website]
related_tasks: [MOB-019]
last_updated: 2026-11-28
---

## Description

Obtainium installs and updates APKs straight from GitHub Releases, with no store review. It works
today; it only needs a link and stable asset names.

## Implementation Steps

- [ ] Add an "Add to Obtainium" link (`obtainium://add/https://github.com/bernardopg/wingdrive`) to the website download page and the README
- [ ] Document the APK filter regex for the per-ABI names (MOB-019) and that pre-releases are opt-in
- [ ] Keep the APK asset names stable across releases

## Acceptance Criteria

- [ ] Obtainium installs the latest APK and detects the next release
