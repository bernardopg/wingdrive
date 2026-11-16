---
id: DESK-098
title: Upload to Google Photos is missing from the menu of a multi-file selection
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M3
sprint: S06
tags: [explorer, google-photos, context-menu, bug]
last_updated: 2026-11-16
---

## Description

Found on 2026-11-16 while recording the INTG-003 demo. With three local photos selected (Ctrl+A), the Explorer context menu has no "Upload to Google Photos"; with one photo it does. The single-file case also never worked before this date: `canUploadToPhotos` checked the extension on `File.name`, which is the stem without extension (fixed in the INTG-003 pull request).

## Implementation Steps

- [ ] Find why `targetFiles` or `canUploadToPhotos` rejects a multi-selection of local photos.
- [ ] Regression test with several selected `File` objects shaped like the daemon sends them.

## Acceptance Criteria

- [ ] Selecting several local photos shows Upload to Google Photos and uploads all of them.
