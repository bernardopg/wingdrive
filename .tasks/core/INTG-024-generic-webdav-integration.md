---
id: INTG-024
title: Generic WebDAV integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
tags: [integrations, webdav, self-hosted]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Many services and NAS devices speak WebDAV: Koofr, Yandex Disk, Seafile, Synology, QNAP, Box (legacy), 4shared. A generic connector with URL, user and password covers them.

## Implementation Steps

1. Connector with server URL, user and password or app password; test the connection before saving.
2. Shares the WebDAV backend with INTG-023.
3. Document tested services and their URLs.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
