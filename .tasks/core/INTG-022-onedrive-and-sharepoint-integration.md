---
id: INTG-022
title: OneDrive and SharePoint integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
tags: [integrations, onedrive, microsoft, oauth]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Microsoft identity platform supports public desktop clients with PKCE and loopback redirects. Microsoft Graph covers personal OneDrive, OneDrive for Business and SharePoint document libraries, with delta queries and resumable upload sessions.

## Implementation Steps

1. Register an app in Microsoft Entra (personal and work accounts), public client, loopback redirect.
2. Scopes `Files.ReadWrite.All`, `offline_access`, `User.Read`; work tenants may need admin consent.
3. Backend over Graph: drive items, `@microsoft.graph.downloadUrl`, upload sessions, `quickXorHash` as revision, recycle bin.
4. Delta queries for change detection.
5. Publisher verification in Entra to avoid the unverified-publisher warning.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
