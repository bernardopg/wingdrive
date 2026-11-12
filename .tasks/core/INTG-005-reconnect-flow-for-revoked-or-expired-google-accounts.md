---
id: INTG-005
title: Reconnect flow for revoked or expired Google accounts
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S06
tags: [integrations, google, oauth, ux]
last_updated: 2026-11-12
related_tasks: [CLOUD-004]
---

## Description

When a refresh fails with `invalid_grant` (access revoked, password changed, testing-mode expiry), Drive listings, sync and uploads fail with a generic error. Users need to see which account broke and fix it with one click.

## Implementation Steps

1. Mark the account as needing reconnection when `GoogleTokenSource` gets `GoogleError::InvalidGrant`; persist the flag in the stored account.
2. Expose it in `IntegrationAccountInfo` and emit an event so the UI updates.
3. Show a banner in Settings > Integrations and a badge on the Drive sidebar location with a Reconnect button that runs the sign-in with `login_hint`.
4. Pause sync pairs of that account and resume them after reconnecting.
5. Map quota (`storageQuotaExceeded`) and rate-limit errors to clear messages.

## Acceptance Criteria

- [ ] Revoking access in the Google account shows a reconnect prompt within one operation.
- [ ] Reconnecting restores Drive and sync without removing the account.
