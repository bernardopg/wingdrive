# Google Drive scope for WingDrive's own client

Decided on 2026-11-16 (INTG-003).

## Context

Google's full Drive scope (`https://www.googleapis.com/auth/drive`) is restricted. An app that asks
for it through a public OAuth client passes verification only with a third-party security
assessment (CASA), paid and renewed every year. WingDrive is free and open source and takes on no
recurring cost for now.

The narrower `drive.file` scope is not sensitive and needs no assessment. With it, Drive returns
only the files WingDrive created or the user opened with WingDrive.

## Decision

- **"Google account" mode** (WingDrive's client, bundled in official builds) asks for `drive.file`.
  It is the mode most people use, and it goes through Google verification with the Photos scopes
  `photoslibrary.appendonly` and `photospicker.mediaitems.readonly` (sensitive) and
  `photoslibrary.readonly.appcreateddata` (not sensitive). `photoslibrary.edit.appcreateddata` was
  dropped on 2026-11-17 because no feature used it.
- **"Google Cloud" mode** (a client the user creates in their own Google Cloud project) keeps the
  full `drive` scope. The project belongs to the user, who signs in as a test user or publishes it
  for themselves, so no verification or assessment applies.
- `GoogleService::Drive` counts as granted with either scope, so accounts connected before this
  change keep working. `IntegrationAccountInfo.drive_access` tells the UI and the CLI which one an
  account has, and the account row explains how to see all of Drive.

## Consequences

- In "Google account" mode, the Drive volume no longer lists the whole of My Drive, and cloud
  folder sync only sees folders WingDrive created.
- Showing existing Drive files in that mode needs the Google Picker, which grants `drive.file`
  access file by file. Done on 2026-11-17: Settings > Integrations > Add files from Google Drive
  opens the Picker, and the volume root lists every granted item whose parent is not granted.
  Picking a folder grants the folder, not its contents, so whole-folder access still needs the
  custom-client mode.
- Moving the bundled client back to full Drive means paying for CASA every year. Revisit if the
  project gets funding.
