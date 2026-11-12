---
id: INTG-020
title: Integration provider framework
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
tags: [integrations, architecture]
last_updated: 2026-11-12
related_tasks: [CLOUD-004]
---

## Description

`IntegrationService` and `IntegrationStore` are written for Google. Each new provider needs the same pieces: an auth method (OAuth with PKCE, OAuth with a fixed client, app passwords, access keys), stored grants, a `VolumeBackend`, a sidebar location and a Settings entry. A shared shape keeps providers small and the UI generic.

## Implementation Steps

1. Define an `IntegrationProvider` trait: id, display name, icon, auth kind, scopes or fields, `mount(account) -> VolumeBackend`, optional services.
2. Generalize `StoredGoogleAccount` into a provider-tagged account and keep Google accounts readable (migration of the stored JSON).
3. Drive the Settings > Integrations page and Add Storage's cloud list from a provider registry query.
4. Move Google onto the trait without behavior changes; keep `google_integration_test.rs` passing.
5. Document how to add a provider in `docs/core/integrations.mdx`.

## Acceptance Criteria

- [ ] Google runs through the framework with no regressions.
- [ ] Adding a provider needs no edits to the Settings page.
