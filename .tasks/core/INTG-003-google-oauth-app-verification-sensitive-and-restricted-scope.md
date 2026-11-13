---
id: INTG-003
title: Google OAuth app verification (sensitive and restricted scopes)
status: In Progress
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, oauth, verification, security]
last_updated: 2026-11-13
related_tasks: [INTG-001, INTG-002]
---

## Description

While the OAuth app is in "Testing", only listed test users can sign in and their grants expire after 7 days. Moving to "In production" without verification shows the "Google hasn't verified this app" screen and caps the app at 100 users. Full Drive access (`https://www.googleapis.com/auth/drive`) is a restricted scope: verification for it needs Google's brand review plus a yearly third-party security assessment (CASA). The Photos scopes are sensitive and need brand and scope review.

Decide the scope strategy before submitting, because it changes the product:

- Keep `drive` (browse all of My Drive as a volume, which is what CLOUD-004 ships) and go through CASA every year.
- Or switch to `drive.file` (not restricted): WingDrive only sees files it created or that the user opens through the Google Picker. Browsing the whole Drive and syncing existing folders would stop working.

## Implementation Steps

1. Record the scope decision as an ADR in `docs/design/`.
2. Prepare the verification package: scope justifications, a demo video of each scope in use, links from INTG-002.
3. Submit brand verification, then sensitive and restricted scope verification.
4. If `drive` stays, run the CASA assessment (tier per Google's request) and schedule the yearly renewal.
5. Publish the app to "In production" once approved.

## Progress (2026-11-13)

The app is "In production" without verification: grants persist, the unverified-app screen shows
and the project is capped at 100 users. Brand verification started with the domain check; a
120x120 logo is in `docs/logo/wingdrive-logo-120.png` for the consent screen. The scope decision
(`drive` with CASA or `drive.file`) is still open.

## Acceptance Criteria

- [ ] Any Google account signs in without the unverified-app warning.
- [ ] Grants no longer expire after 7 days.
- [ ] The yearly renewal date is tracked.
