---
id: INTG-025
title: iCloud Drive integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
sprint: S13
tags: [integrations, icloud, apple, macos, windows]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

Apple offers no public iCloud Drive API for third-party apps outside Apple platforms. On macOS iCloud Drive is a local folder backed by File Provider (`~/Library/Mobile Documents/com~apple~CloudDocs`); on Windows, iCloud for Windows creates a local iCloud Drive folder. Linux has only unofficial, reverse-engineered clients that break with 2FA changes.

## Implementation Steps

1. macOS: detect the iCloud Drive folder and add it as a location with cloud status (downloaded, cloud-only) from file attributes; offer "Download now" and "Remove download".
2. Windows: detect the iCloud Drive folder of iCloud for Windows the same way.
3. Linux: record the decision (not supported, or an opt-in experimental client) in an ADR.
4. iCloud Photos: research the same constraints.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
