---
id: INTG-004
title: End-to-end validation with real Google on every platform
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, testing, linux, windows, macos, blocked]
last_updated: 2026-11-28
related_tasks: [CLOUD-004, TAURI-012]
---

## Description

CLOUD-004 was verified against a fake Google (`core/tests/google_integration_test.rs`) and an isolated `wing-server`. It has never talked to Google itself or run in a packaged app.

## Implementation Steps

1. With the INTG-001 client, sign in from the Linux packages (deb, AUR, AppImage), Windows (`.exe`) and the `wing-server` web UI; macOS when TAURI-012 has a machine.
2. Check the loopback redirect through the desktop firewall on Windows and inside the Flatpak sandbox.
3. Run the matrix: two accounts, Drive browse with duplicates and Google Docs, upload and download over 1 GB with a dropped connection, rename and move, trash, local editing with a conflict, two-way sync with deletions, Photos upload to an album, picker import, disconnect.
4. Revoke access from https://myaccount.google.com/permissions and confirm WingDrive asks to reconnect (INTG-005).
5. Record results in the task log and file bugs as DESK/INTG tasks.

## Log

- 2026-11-13: first real round trip. The author connected their Google account from the desktop
  app of branch `feat/integrations-connection-modes` (debug build, Linux, isolated instance) with
  the bundled INTG-001 client in the "Google account" mode. The rest of the matrix is pending.
- 2026-11-22 (DEV-008): moved to `To Do` during S05 board cleanup. Blocked on third-party/external
  access: real Google validation on Windows/macOS packages and remaining matrix need external
  accounts/machines; not actively in progress.
- 2026-11-28: second round trip, Linux, packaged `wingdrive-bin` 3.8.2 (AUR, `/opt/wingdrive`),
  isolated instance (`--data-dir` on disk, `--instance gtest`, not the author's daemon), bundled
  client (`google-client show` reports `Bundled`), author's Google account.
  - `integrations connect google --no-browser`, consent in Chrome: account chooser and consent
    screen show "WingDrive" with the privacy and terms links and no unverified-app warning.
    Scopes requested: `openid email profile drive.file photoslibrary.appendonly
    photoslibrary.readonly.appcreateddata photospicker.mediaitems.readonly`.
  - `integrations list`: one Google account, services Drive (`app_files`) and Photos, sign-in
    "WingDrive app", `drive_picker: true`, `needs_reconnect: false`.
  - Drive mounted on connect; `file list gdrive://<account>/` listed the root (1 item, the files
    granted to WingDrive).
  - `integrations photos albums`: call succeeded, 0 albums created by WingDrive in this account.
  - Drive Picker (`integrations.google.drive.picker.start`) with a Picker-only API key restricted to
    the referrer `http://127.0.0.1/*`: the Picker loaded on the loopback page and listed Drive;
    cancelled, status `cancelled`.
  - `integrations remove --yes` revoked the grant: WingDrive no longer appears in the account's
    third-party connections. Instance stopped and its data deleted. No key or client secret in the
    daemon log.

## Acceptance Criteria

- [ ] Every step of the matrix passes on Linux and Windows packages.
- [ ] Results recorded with versions and dates.
