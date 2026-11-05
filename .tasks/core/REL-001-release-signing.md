---
id: REL-001
title: Signed releases with WingDrive-owned credentials
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S05
tags: [release, signing, updater, ci]
related_tasks: [MOB-009, TAURI-012]
last_updated: 2026-11-05
---

## Description

The repository already builds, tests and publishes on its own: CI, the `v*` release
(AppImage, deb, AUR `wingdrive-bin`, and since `v3.0.0` the Windows `.exe` and `.msi`)
and the GHCR server image run with this repository's permissions only. What is still unsigned needs credentials that only
the author can create.

This task takes over the signing half of the old independence task (FORK-002,
removed on 2026-11-02; the history is in git).

## Acceptance Criteria

- [ ] Tauri updater key generated; the public key in `tauri.conf.json` and the private key in repository secrets
- [ ] A tag run publishes signed updater artifacts and `latest.json`
- [ ] Apple signing and notarization plan written (Developer ID, secrets, workflow step)
- [ ] Windows Authenticode signing plan written for the `.exe` and `.msi` from TAURI-016
- [ ] Google Play upload key and the four `WINGDRIVE_ANDROID_KEYSTORE_*` secrets in place (with MOB-009)
- [ ] One tag run signs every artifact that has credentials and skips the others with a warning
- [ ] Alpha, beta and rc tags do not publish `wingdrive-bin` on AUR (the `aur` job runs for every
      tag today, so a pre-release would reach every AUR user)

## Notes

- `release.yml` already builds the Android bundle without uploading when the keystore
  secrets are missing, so a desktop release never waits on Google Play.
- The `v3.0.0` Windows installers are unsigned: SmartScreen warns on first run, and Smart App
  Control may block them (TAURI-016). The UAC helper of TAURI-017 also needs the certificate.
- `wing update` (CLI-002) should download only signed artifacts, if it downloads at all.
