---
id: INTG-030
title: "Object storage from the UI: S3, R2, B2, Wasabi, MinIO, GCS, Azure"
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
tags: [integrations, s3, object-storage, gcs, azure]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

CLOUD-003 added these as volumes through OpenDAL with access keys, but they were never validated end to end and do not appear in Settings > Integrations.

## Implementation Steps

1. Move them onto the provider framework with a connection test before saving.
2. Validate each against a real or local endpoint (MinIO in CI).
3. Native Backblaze B2 API where S3 compatibility falls short.
4. Show bucket usage where the service reports it.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
