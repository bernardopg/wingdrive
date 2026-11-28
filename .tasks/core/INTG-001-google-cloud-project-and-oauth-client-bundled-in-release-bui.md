---
id: INTG-001
title: Google Cloud project and OAuth client bundled in release builds
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, oauth, release, ci, blocked]
last_updated: 2026-11-28
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

> 2026-11-22 (DEV-008): moved to `To Do` during S05 board cleanup. Bundled client and secrets are
> in place; remaining work (AUR nightly, Docker, Flatpak, mobile, CI client-source check) is paused,
> not actively in progress.

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

- `WINGDRIVE_GOOGLE_CLIENT_ID` and `WINGDRIVE_GOOGLE_CLIENT_SECRET` set as repository secrets;
  v3.3.0 is the first release built with them.

## Progress (2026-11-28)

Checked in Google Cloud Console (Google Auth Platform > Branding):

- The terms link https://wingdrive.org/service-terms is set, next to the homepage, the privacy
  policy and the authorized domain `wingdrive.org`. Google's account chooser and consent screen
  link both pages.
- No logo is uploaded, on purpose. The Branding page states that uploading a logo requires
  submitting the app for verification again. Branding and Data Access are both verified today, so
  a logo would start a new brand review for no functional gain. Upload
  `docs/logo/wingdrive-logo-120.png` only when another verification round is needed anyway.
- The bundled client of the packaged 3.8.2 build reports source `Bundled`
  (`wing integrations google-client show`) and signed in against real Google (INTG-004).
- API keys: "WingDrive Picker" (2026-11-17, Google Picker API only, no application restriction)
  is the one in the `WINGDRIVE_GOOGLE_API_KEY` repository secret. A second key, "WingDrive Picker
  (loopback) 2026-11-28", is restricted to the Google Picker API and to the website referrer
  `http://127.0.0.1/*`, which matches the Picker page on any loopback port. The Picker loaded and
  listed Drive with it on 2026-11-28. The author keeps it outside the repository.

Logo, later the same day: the author chose to upload it after all. `docs/logo/wingdrive-logo-120.png`
(PNG, 120x120, 4.9 KB, within Google's square 120 px and 1 MB limits) was uploaded on the Branding
page and the brand was submitted with "Verificar marca". The automated check passed in about five
minutes ("Sua marca foi verificada, mas ainda não está sendo mostrada aos usuários"), and
"Publicar branding" made it live: Branding now reads "Sua marca foi verificada e está aparecendo
para os usuários", with the logo as "Logotipo do app atual". Scopes, clients, secrets and API
keys were not changed. INTG-003 has the verification record.

Left: AUR `wingdrive-nightly` (builds from source), Docker image, Flatpak and mobile, and the CI
check of the client source.

## Acceptance Criteria

- [ ] A release build shows "Client ... included in this build" in Settings > Integrations.
- [ ] Building without the secrets still compiles and asks for a client.
- [ ] The client secret never appears in logs, crash reports or `integrations.google.client` output.

## API key (2026-11-28)

The repository secret `WINGDRIVE_GOOGLE_API_KEY` now holds a key restricted to the Google Picker
API and `http://127.0.0.1/*` (created 2026-11-28; a local copy is outside the repository). Releases
up to 3.8.2 embed the older 2026-11-17 key, which has no application restriction. Left: once those
builds are out of use, delete that key or restrict it the same way in the Cloud Console
(APIs & Services > Credentials, project `wingdrive-514618`).

