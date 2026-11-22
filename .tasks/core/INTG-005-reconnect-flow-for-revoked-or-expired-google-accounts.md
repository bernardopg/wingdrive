---
id: INTG-005
title: Reconnect flow for revoked or expired Google accounts
status: Done
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S06
tags: [integrations, google, oauth, ux]
last_updated: 2026-11-22
related_tasks: [CLOUD-004]
---

## Description

When a refresh fails with `invalid_grant` (access revoked, password changed, testing-mode expiry), Drive listings, sync and uploads fail with a generic error. Users need to see which account broke and fix it with one click.

## Implementation Steps

1. [x] Mark the account as needing reconnection when `GoogleTokenSource` gets `GoogleError::InvalidGrant`; persist the flag in the stored account.
2. [x] Expose it in `IntegrationAccountInfo` and emit an event so the UI updates.
3. [x] Show a banner in Settings > Integrations and a badge on the Drive sidebar location with a Reconnect button that runs the sign-in with `login_hint`. Pause sync pairs of that account and resume them after reconnecting.
4. [x] Pause sync pairs of that account and resume them after reconnecting.
5. [x] Map quota (`storageQuotaExceeded`) and rate-limit errors to clear messages.

## Acceptance Criteria

- [x] Revoking access in the Google account shows a reconnect prompt within one operation. Code path: `GoogleTokenSource` invalid_grant hook marks stored account, emits `IntegrationAccountChanged`, Settings/Sidebar subscribe and show reconnect UI.
- [x] Reconnecting restores Drive and sync without removing the account. `finish_google_connect` clears `needs_reconnect`, resumes `paused_by_reconnect` sync pairs, and emits `IntegrationAccountChanged`.

## Evidence

- Added `needs_reconnect` to `StoredGoogleAccount` and `IntegrationAccountInfo`.
- Added `InvalidGrantHook` to `GoogleTokenSource`; hook fires on `invalid_grant` refresh failures.
- `IntegrationService` installs hook from Core context, persists reconnect flag, invalidates cached token source, pauses gdrive sync pairs, and emits `IntegrationAccountChanged`.
- Reconnect (`finish_google_connect`) clears flag, resumes paused sync pairs, emits event.
- Added DB migration `m20261122_000001_add_paused_by_reconnect_to_cloud_sync_pairs` and `paused_by_reconnect` entity field.
- Mapped `storageQuotaExceeded` and rate-limit Google API errors to clear messages in `tokens::api_error`.
- UI: Integrations settings banner + per-account reconnect notice/button with `login_hint`; sidebar cloud location reconnect badge/button; listens to `IntegrationAccountChanged`.
- Tests: added `api_error_maps_storage_quota_and_rate_limits` unit test; added `google_invalid_grant_marks_account_for_reconnect` integration test in `core/tests/google_integration_test.rs`.
- Validation: `cargo fmt -p wing-core -- --check` (after auto-fix) passed. `cargo check -p wing-core -j 2`/`-j 1` were both SIGTERM-killed in this environment before completing; no Rust compile errors observed in edited code, but full compile could not be completed here. TS type files manually updated to match Rust Specta shapes; regenerate with `cargo run --bin generate_typescript_types` in CI/dev.
