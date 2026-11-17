---
id: REL-001
title: Signed releases with WingDrive-owned credentials
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S06
tags: [release, signing, updater, ci]
related_tasks: [MOB-009, TAURI-012]
last_updated: 2026-11-17
---

## Description

The repository already builds, tests and publishes on its own: CI, the `v*` release
(AppImage, deb, AUR `wingdrive-bin`, and since `v3.0.0` the Windows `.exe` and `.msi`)
and the GHCR server image run with this repository's permissions only. What is still unsigned needs credentials that only
the author can create.

This task takes over the signing half of the old independence task (FORK-002,
removed on 2026-11-02; the history is in git).

## Acceptance Criteria

- [x] Tauri updater key generated; the public key in `tauri.conf.json` and the private key in repository secrets (2026-11-16)
- [ ] A tag run publishes signed updater artifacts and `latest.json`
- [x] Apple signing and notarization plan written (Developer ID, secrets, workflow step)
- [x] Windows Authenticode signing plan written for the `.exe` and `.msi` from TAURI-016
- [ ] Google Play upload key and the four `WINGDRIVE_ANDROID_KEYSTORE_*` secrets in place (with MOB-009)
- [ ] One tag run signs every artifact that has credentials and skips the others with a warning
- [x] Alpha, beta and rc tags do not publish `wingdrive-bin` on AUR. They go to
      `wingdrive-nightly`, which stable tags update too (`aur.py`, the `aur` job matrix; 2026-11-05)

## Notes

- `release.yml` already builds the Android bundle without uploading when the keystore
  secrets are missing, so a desktop release never waits on Google Play.
- The `v3.0.0` Windows installers are unsigned: SmartScreen warns on first run, and Smart App
  Control may block them (TAURI-016). The UAC helper of TAURI-017 also needs the certificate.
- `wing update` (CLI-002) should download only signed artifacts, if it downloads at all.

## Updater (2026-11-16)

- Key pair from `tauri signer generate`. Private key and password in the repository secrets
  `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, and in the author's
  Bitwarden vault, item "WingDrive Tauri updater signing key" (checked by reading it back). The
  public key and the endpoint `releases/latest/download/latest.json` are in `tauri.conf.json`.
- Only the AppImage (`$APPIMAGE` set) and the Windows installers update themselves. The AUR
  packages run the extracted AppImage from `/opt/wingdrive` without `$APPIMAGE`, and the `.deb` is
  owned by apt, so both report "package manager" and never download anything: `pacman`, `yay` and
  `apt` stay the only writers of their files, with no conflict on the next `-Syu`.
- `release.yml` and `windows-bundle.yml` build tagged bundles with `createUpdaterArtifacts` when the
  key is present and ship the `.sig` files; the publish job writes `latest.json` with the signed
  artifacts only (`scripts/release/updater_manifest.py`).
- Settings > General > Updates: channel, "Check for updates", "Install and restart" with download
  progress, and "Check for updates automatically" (one check 30 s after start, desktop notification).
- Left: the first tag that publishes `latest.json`, then an AppImage of the previous version updating
  itself to it.

## Authenticode plan (Windows)

1. Buy an OV code-signing certificate for "WingDrive" (EV is no longer needed for SmartScreen
   reputation since 2024; OV builds reputation with downloads). Certificates now live on a hardware
   token or a cloud HSM, so use a cloud signing service that GitHub Actions can reach (Azure Trusted
   Signing is the cheapest, about USD 10 a month, and works with `signtool` through its dlib).
2. Secrets: `AZURE_TENANT_ID`, `AZURE_CLIENT_ID`, `AZURE_CLIENT_SECRET`, plus the account and profile
   names. Tauri signs through `bundle.windows.signCommand`, which runs `trusted-signing-cli` (or
   `signtool sign /dlib ...`) on every binary and installer it builds, so `wing-daemon.exe`, `wing.exe`,
   the NSIS `.exe` and the `.msi` are all signed.
3. `windows-bundle.yml` passes the signing command only on tagged releases with the secrets present;
   pull requests keep building unsigned.
4. Check on a clean Windows machine that SmartScreen names the publisher and that Smart App Control
   lets the installer run (TAURI-012), then sign the UAC helper of TAURI-017 with the same profile.

## Apple plan (macOS)

1. Apple Developer Program membership (USD 99 a year) and a "Developer ID Application" certificate,
   exported as `.p12`.
2. Secrets: `APPLE_CERTIFICATE` (base64 `.p12`), `APPLE_CERTIFICATE_PASSWORD`,
   `APPLE_SIGNING_IDENTITY`, and for notarization `APPLE_API_KEY`, `APPLE_API_ISSUER` and the `.p8`
   key (App Store Connect API key), which Tauri's bundler reads to run `notarytool` and staple.
3. A macOS job in `release.yml` (macos-14, universal or arm64 plus x86_64) builds the `.dmg`, signs
   with the hardened runtime and the existing `Entitlements.plist`, notarizes and staples; the
   updater artifact for macOS (`.app.tar.gz` and `.sig`) joins `latest.json` as `darwin-aarch64` and
   `darwin-x86_64`.
4. Run the TAURI-012 matrix on a Mac before the first signed macOS release.

## Authenticode in the workflow (2026-11-17)

#422 does step 3 of the plan: `windows-bundle.yml` signs a tagged release through Azure Artifact
Signing (`artifact-signing-cli` 0.12.0 as Tauri's `signCommand`) when the six secrets listed in
`docs/core/releases.mdx` exist, and fails the run when an installer has no valid signature. Without
them the run warns and ships unsigned installers. Left for the author: the Azure account,
certificate profile and secrets, then one signed tag run checked on a clean machine.
