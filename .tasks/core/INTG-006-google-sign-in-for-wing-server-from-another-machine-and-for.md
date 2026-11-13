---
id: INTG-006
title: Google sign-in for wing-server from another machine and for mobile
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
sprint: S12
tags: [integrations, google, oauth, server, mobile]
last_updated: 2026-11-12
related_tasks: [INTG-001, MOB-000]
---

## Description

The loopback redirect lands on `127.0.0.1` of the machine that runs the daemon. That works for the desktop app, but not when the web UI of a `wing-server` is opened from another computer, and not on Android or iOS where the core is embedded. Google's device code flow does not allow Drive scopes.

## Implementation Steps

1. wing-server: add a Web application OAuth client, an HTTPS `/oauth/google/callback` route on the server, and use it when the request did not come from loopback.
2. Android and iOS: use the platform client types with a custom URI scheme or app link and PKCE (AppAuth pattern), passing the code to the embedded core.
3. Share account storage and token refresh with the desktop path.

## Acceptance Criteria

- [ ] A user opening wing-server's web UI from a laptop connects Google Drive.
- [ ] The Android app connects a Google account.
