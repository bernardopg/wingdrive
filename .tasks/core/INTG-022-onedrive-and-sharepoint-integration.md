---
id: INTG-022
title: OneDrive and SharePoint integration
status: In Progress
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, onedrive, microsoft, oauth]
last_updated: 2026-11-13
related_tasks: [INTG-020, INTG-010]
---

## Description

Microsoft identity platform supports public desktop clients with PKCE and loopback redirects. Microsoft Graph covers personal OneDrive, OneDrive for Business and SharePoint document libraries, with delta queries and resumable upload sessions.

## Implementation Steps

1. Register an app in Microsoft Entra (personal and work accounts), public client, loopback redirect.
2. Scopes `Files.ReadWrite.All`, `offline_access`, `User.Read`; work tenants may need admin consent.
3. Backend over Graph: drive items, `@microsoft.graph.downloadUrl`, upload sessions, `quickXorHash` as revision, recycle bin.
4. Delta queries for change detection.
5. Publisher verification in Entra to avoid the unverified-publisher warning.

## Progress (2026-11-13)

Done in code with INTG-010: sign-in as a public client with PKCE through the shared loopback
listener (`http://localhost:<port>/oauth/callback`), scopes `Files.ReadWrite`, `User.Read`,
`offline_access`, both connection modes, OneDrive mounted as `onedrive://<email>/` with a sidebar
location, rotated refresh tokens saved back to the library, CLI `wing integrations connect onedrive`
and `microsoft-client`, and `core/tests/microsoft_integration_test.rs` against a fake identity
platform. File operations go through OpenDAL's OneDrive service with an access token only.

Left: register WingDrive's app in Microsoft Entra and pass `WINGDRIVE_MICROSOFT_CLIENT_ID` to
release builds (as INTG-001 does for Google), a round trip against the real OneDrive, SharePoint
libraries and "Shared with me", delta queries, and publisher verification.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [x] Several accounts per library.
- [x] Docs in `docs/core/integrations.mdx`; website still to do.
