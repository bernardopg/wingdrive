---
id: INTG-031
title: SFTP and SMB as cloud-style volumes
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
tags: [integrations, sftp, smb, network]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

DESK-009 mounts SMB and SFTP through GIO/gvfs on Linux. A cross-platform connector through OpenDAL (`services-sftp`) or `smb` crates would make them work on Windows and macOS and in cloud sync.

## Implementation Steps

1. Decide between gvfs (Linux only) and native connectors, recorded with DESK-009.
2. Key-based SFTP and SMB credentials through the framework.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
