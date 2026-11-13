---
id: INTG-021
title: Dropbox integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S11
tags: [integrations, dropbox, oauth]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Dropbox supports OAuth 2 with PKCE for public clients, so no secret ships in the binary. API v2 offers `content_hash` for change detection, upload sessions for large files and a cursor-based change feed (`list_folder/continue`). OpenDAL's dropbox service is compiled in but needs a pasted refresh token today.

## Implementation Steps

1. Register the WingDrive app in the Dropbox App Console (full Dropbox access), with the loopback redirect.
2. Sign-in with PKCE through the framework; several accounts.
3. Backend over API v2 or OpenDAL with the obtained refresh token; map `content_hash` to `revision`.
4. Large uploads with upload sessions; trash maps to Dropbox's deleted files (restorable).
5. Apply for production status in the Dropbox App Console (needed past 500 users).

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
