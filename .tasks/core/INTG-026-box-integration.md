---
id: INTG-026
title: Box integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
tags: [integrations, box, oauth]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Box API with OAuth 2 (user authentication), upload sessions for files over 50 MB, events API for changes, trash.

## Implementation Steps

1. Register the app in the Box Developer Console.
2. Sign-in, backend, trash and change feed through the framework.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
