---
id: INTG-001
title: Google Cloud project and OAuth client bundled in release builds
status: In Progress
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, oauth, release, ci]
last_updated: 2026-11-13
related_tasks: [CLOUD-004, REL-001]
---

## Description

Today Google sign-in only works when the user brings an OAuth client (Settings > Integrations > Google sign-in client) or the daemon runs with `WINGDRIVE_GOOGLE_CLIENT_ID`/`WINGDRIVE_GOOGLE_CLIENT_SECRET`. For "just sign in", every release must ship WingDrive's own client, read at build time through `option_env!` in `core/src/service/integrations/mod.rs`.

Google treats a "Desktop app" client secret as not confidential, so embedding it in the binary is the documented model for installed apps. PKCE protects the code exchange.

## Implementation Steps

1. Create the `wingdrive` project in Google Cloud Console under the author's account.
2. Enable Google Drive API, Photos Library API and Photos Picker API.
3. Configure the OAuth consent screen: app name WingDrive, logo, support email, homepage https://wingdrive.org, privacy policy and terms (INTG-002), authorized domain `wingdrive.org`.
4. Add the scopes the app requests (identity, `drive`, the four Photos scopes) to the consent screen.
5. Create an OAuth client of type Desktop app.
6. Store `WINGDRIVE_GOOGLE_CLIENT_ID` and `WINGDRIVE_GOOGLE_CLIENT_SECRET` as GitHub Actions secrets.
7. Pass them as environment variables to every build that compiles `wing-core` for distribution: `release.yml` (deb, AppImage, Windows `.exe`/`.msi`), the AUR `wingdrive-bin`/`wingdrive-nightly` packaging, the server Docker image, Flatpak, and the mobile core.
8. Keep forks and local builds working without the secrets (the user-supplied client path stays).
9. Add a CI check that release artifacts report `integrations.google.client` source `bundled`.

## Progress (2026-11-13)

- Project `wingdrive-514618` created under the author's account, with Google Drive API, Photos
  Library API and Photos Picker API enabled.
- Consent screen: External, app name WingDrive, homepage https://www.wingdrive.org, privacy policy
  https://wingdrive.org/privacy, authorized domain `wingdrive.org`, the eight scopes the app
  requests. Published "In production" (unverified, 100-user cap) so grants do not expire after
  7 days.
- Desktop client "WingDrive Desktop" created. The author keeps the ID and secret outside the
  repository (`~/.config/wingdrive-dev/oauth.env`).
- `release.yml` and `windows-bundle.yml` pass `WINGDRIVE_GOOGLE_CLIENT_ID`,
  `WINGDRIVE_GOOGLE_CLIENT_SECRET` and `WINGDRIVE_MICROSOFT_CLIENT_ID` from repository secrets to
  the builds; without the secrets the build still compiles.
- Signing in with the bundled client worked against real Google from the Linux desktop app
  (isolated instance, debug build), INTG-004.

Left: add the GitHub secrets, logo and terms link on the consent screen, AUR `wingdrive-nightly`
(builds from source), Docker image, Flatpak and mobile, and the CI check of the client source.

## Acceptance Criteria

- [ ] A release build shows "Client ... included in this build" in Settings > Integrations.
- [ ] Building without the secrets still compiles and asks for a client.
- [ ] The client secret never appears in logs, crash reports or `integrations.google.client` output.
