---
id: INTG-027
title: pCloud integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
sprint: S14
tags: [integrations, pcloud, oauth]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

pCloud has an OAuth 2 API with separate US and EU data regions, checksums and a diff API for changes.

## Implementation Steps

1. Register the app with pCloud.
2. Handle the account's region host.
3. Backend, trash (rewind) and change feed.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
