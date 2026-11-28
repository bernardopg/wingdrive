---
id: DIST-003
title: "Google Play: upload the AAB to the internal track from the release workflow"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S10
tags: [android, google-play, ci, release]
related_tasks: [DIST-001, MOB-009]
last_updated: 2026-11-28
---

## Description

`.github/workflows/mobile.yml` already builds a signed AAB on tags. Add a step that uploads it
to Play so a release needs no manual upload. Production promotion stays a manual click.

## Implementation Steps

- [ ] Create a Google Cloud service account with the Play Developer API and grant it release rights in Play Console
- [ ] Store its JSON key as the secret `PLAY_SERVICE_ACCOUNT_JSON`
- [ ] In `mobile.yml`, upload the AAB and the release notes (Fastlane changelog for the versionCode) to the internal track on stable tags; alpha, beta and rc tags go to the closed track
- [ ] Skip the step with a warning when the secret is missing, like the keystore step
- [ ] Document the step in `docs/core/releases.mdx`

## Acceptance Criteria

- [ ] A stable tag puts its AAB on the internal track with release notes
- [ ] A run without the secret still finishes and warns
