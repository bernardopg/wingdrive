---
id: INTG-001
title: Google Cloud project and OAuth client bundled in release builds
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S06
tags: [integrations, google, oauth, release, ci]
last_updated: 2026-11-12
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

## Acceptance Criteria

- [ ] A release build shows "Client ... included in this build" in Settings > Integrations.
- [ ] Building without the secrets still compiles and asks for a client.
- [ ] The client secret never appears in logs, crash reports or `integrations.google.client` output.
