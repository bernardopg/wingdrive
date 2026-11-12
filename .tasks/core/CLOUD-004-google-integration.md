---
id: CLOUD-004
title: Google integration (Drive, Photos and cloud folder sync)
status: In Progress
assignee: bernardopg
parent: CLOUD-000
priority: High
milestone: M5
sprint: S05
tags: [cloud, integrations, google-drive, google-photos, oauth, sync]
last_updated: 2026-11-12
related_tasks: [CLOUD-003, FILE-003]
---

## Description

First integration with a third-party cloud: connect several Google accounts
per library through the browser, mount each Google Drive as a cloud volume,
copy, move, rename, edit and sync files with it, and upload to and import
from Google Photos. Requested by the author on 2026-11-12. Design and
behavior are documented in `docs/core/integrations.mdx`.

## Implementation Steps

1. OAuth for installed apps with PKCE and a loopback redirect; encrypted
   account storage per library; shared access tokens with retries.
2. `GoogleDriveBackend`: path addressing over Drive IDs, Google Docs export,
   resumable uploads, ranged downloads, rename, copy, trash, quota.
3. Copy, move and rename jobs between local and cloud volumes (FILE-003);
   live cloud folder listing; create folder and trash on cloud volumes.
4. Cloud sync pairs (two-way, upload, download) with manifest, conflict
   copies, trash on deletion and a scheduler.
5. Local editing of cloud files with upload on save and conflict copies.
6. Google Photos upload into app albums and picker import.
7. Settings > Integrations, Explorer actions, Add Storage sign-in,
   `wing integrations`, docs and tests.

## Acceptance Criteria

- [x] Two Google accounts connect to one library through the browser and each gets its own Drive volume and sidebar location.
- [x] Drive folders list in the Explorer, including duplicate names and Google Docs.
- [x] Files and folders upload, download, copy, move and rename between local folders and Drive.
- [x] A Drive file opens in a local app and saved edits upload back; a remote change yields a conflict copy.
- [x] A local folder syncs with a Drive folder in all three modes, on demand and on an interval.
- [x] Photos upload to Google Photos into an album, and picked photos download at original quality.
- [x] Disconnecting revokes the grant and removes only that account's Drive.
- [x] The Drive volume survives a daemon restart and the volume monitor's refresh.
- [x] Integration test against a fake Google (`core/tests/google_integration_test.rs`) and runtime check on an isolated `wing-server`.
- [ ] Sign-in and one round trip against real Google with a WingDrive OAuth client (needs the author's Google Cloud project; see "Next").
- [ ] Checked in the packaged desktop app.

## Next

- Create the WingDrive Google Cloud project, enable Drive API, Photos Library
  API and Photos Picker API, and decide whether to bundle the client ID at
  build time (`WINGDRIVE_GOOGLE_CLIENT_ID`). Publishing past testing mode
  needs Google's verification for the restricted `drive` scope.
- Shared drives and "Shared with me" are not browsed.
- Drive files are not indexed, so they do not appear in library search.

## Log

- 2026-11-12: implemented on `feat/google-integration`. Runtime check on an
  isolated `wing-server` with a fake Google found two bugs, both fixed: the
  volume monitor removed cloud volumes 30 s after startup, and
  `locations.list` turned cloud locations into local paths.
