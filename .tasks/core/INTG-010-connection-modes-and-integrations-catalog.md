---
id: INTG-010
title: Connection modes and integrations catalog
status: Done
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, onedrive, oauth, ui]
last_updated: 2026-11-13
related_tasks: [INTG-001, INTG-022, CLOUD-004]
---

## Description

Users should connect a service by only signing in to their account, and still be able to use a
client of their own. Every Google or Microsoft sign-in needs an OAuth client registered with the
provider, so "only sign in" means WingDrive's own client, included in release builds (INTG-001 for
Google), while the other mode uses a client the user registers in Google Cloud or Microsoft Entra.

Settings > Integrations becomes a catalog of large cards, one per service and mode, with the
service's single-color logo, title and modality. Selecting a card explains the mode's advantages,
disadvantages and setup steps and connects an account. OneDrive is the second service, so both
modes are exercised by two providers.

## Implementation Steps

1. `ConnectionMode` (`app`, `custom`) on the Google and Microsoft connect actions and on every
   account; `integrations.google.client` and `integrations.microsoft.client` return both clients.
2. Microsoft sign-in as a public client with PKCE, rotating token source, `OneDriveBackend` over
   OpenDAL, mount, restore and removal.
3. Catalog UI, per-mode panel, connected accounts for both providers, cloud sync with OneDrive.
4. CLI: `connect google --own-client`, `connect onedrive`, `microsoft-client`.
5. Tests: Microsoft integration test against a fake, store and OAuth unit tests, settings page test.

## Acceptance Criteria

- [x] Each service shows one card per mode with logo, title, modality and status.
- [x] Selecting a card shows advantages, disadvantages and steps, and connects in that mode.
- [x] Accounts show the mode they signed in with; old Google accounts get one inferred.
- [x] OneDrive connects in both modes and mounts as `onedrive://<email>/`.
- [x] English and Portuguese strings; docs in `docs/core/integrations.mdx`.
- [x] Checked in an isolated `wing-server` (dark, light, phone width).
