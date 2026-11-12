---
id: INTG-029
title: Proton Drive integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
tags: [integrations, proton, encryption]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Proton Drive is end-to-end encrypted and Proton is publishing official SDKs. There is no stable public API for third parties yet.

## Implementation Steps

1. Track Proton's SDK release and terms.
2. Prototype once an official SDK allows third-party clients.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
