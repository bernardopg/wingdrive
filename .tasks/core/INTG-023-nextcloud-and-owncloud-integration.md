---
id: INTG-023
title: Nextcloud and ownCloud integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S11
tags: [integrations, nextcloud, owncloud, webdav, self-hosted]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Nextcloud exposes files over WebDAV and offers Login Flow v2, which hands the app an app password after the user signs in in the browser. No developer registration is needed, so any server works.

## Implementation Steps

1. Ask for the server URL, run Login Flow v2 in the browser, store the app password.
2. Enable OpenDAL `services-webdav` or write a WebDAV backend; use `oc:fileid` and ETag as revision.
3. Trash through Nextcloud's trashbin, chunked uploads (v2) for large files.
4. Optional: Nextcloud Photos/Memories as a photo source.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
