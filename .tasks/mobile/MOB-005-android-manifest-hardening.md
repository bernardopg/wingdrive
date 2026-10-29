---
id: MOB-005
title: "Android manifest hardening: backup and unused permissions"
status: In Progress
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S04
tags: [mobile, android, security]
last_updated: 2026-10-28
---

## Description

`allowBackup="true"` sends the core data directory, with the device keys and id, to Google backup.
Restoring it on a new phone creates a second peer with the same identity. The manifest also asks for
`RECORD_AUDIO` and `SYSTEM_ALERT_WINDOW`, which the app does not use; the camera only reads QR codes.

## Acceptance Criteria

- [x] `allowBackup` is `false` in `app.json` and in the generated manifest
- [x] `RECORD_AUDIO` and `SYSTEM_ALERT_WINDOW` removed from the release manifest
- [ ] QR pairing still opens the camera

## Evidence

- `app.json`: `allowBackup: false`, `blockedPermissions` for both; generated manifest updated with
  `tools:node="remove"`. Merged debug manifest: `allowBackup="false"`, no `RECORD_AUDIO`.
  `SYSTEM_ALERT_WINDOW` stays only in `src/debug/AndroidManifest.xml` (React Native dev overlay).
- Pending: QR pairing camera check on the Moto G7.
