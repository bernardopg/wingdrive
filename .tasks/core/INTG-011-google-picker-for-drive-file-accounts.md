---
id: INTG-011
title: Google Picker for drive.file accounts
status: Done
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, drive, oauth]
last_updated: 2026-11-17
related_tasks: [INTG-003, INTG-007, CLOUD-004]
---

## Description

Since 3.4.0 WingDrive's own Google client asks for `drive.file` (ADR
`docs/design/GOOGLE-DRIVE-SCOPE.md`), so the Drive volume only showed files WingDrive created.
Existing files were unreachable without a Google Cloud client of the user's own. The Google Picker
adds files to the `drive.file` grant one by one, without a restricted scope or a yearly CASA
assessment.

## Implementation Steps

1. Serve a Picker page on a random loopback port with the account's access token, a browser API
   key (`WINGDRIVE_GOOGLE_API_KEY`) and the Cloud project number as app ID; the page reports back
   how many items were picked (`google/drive_picker.rs`).
2. `integrations.google.drive.picker.start` and `.status`; a pick refreshes the account's Drive
   listings.
3. In `drive.file` mode, list every granted item whose parent is not granted at the volume root, so
   picked files are reachable.
4. Settings > Integrations: "Add files from Google Drive" on accounts with `drive_picker`.
5. Drop `photoslibrary.edit.appcreateddata`, which no feature used (least privilege for INTG-003).
6. Build the API key into release builds (GitHub secret `WINGDRIVE_GOOGLE_API_KEY`).

## Acceptance Criteria

- [x] An account connected with WingDrive's client picks existing Drive files and sees them in its
      Drive volume.
- [x] Custom-client accounts (full `drive`) do not show the Picker.
- [x] The bundled client asks only for the scopes a feature uses.
