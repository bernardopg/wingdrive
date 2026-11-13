---
id: INTG-000
title: "Epic: Third-party cloud integrations"
status: To Do
assignee: bernardopg
priority: High
milestone: M5
sprint: S10
tags: [epic, cloud, integrations]
last_updated: 2026-11-12
related_tasks: [CLOUD-004, CLOUD-003, FILE-003]
---

## Description

Connect accounts from third-party storage services to a library and expose them through WingDrive's own concepts: a cloud volume the Explorer, copy jobs and cloud sync already understand, plus service-specific jobs for services without a file tree. Google (CLOUD-004) is the first provider. This epic tracks the remaining Google work needed for any user to connect with a single sign-in, and the next providers.

The goal for every provider: install WingDrive, open Settings > Integrations, sign in, and use the service. No developer console, no pasted tokens.

## Implementation Steps

1. Finish Google for every user: INTG-001 to INTG-008.
2. Provider framework so new services plug in the same way: INTG-020.
3. Providers, in order of demand: INTG-021 (Dropbox), INTG-022 (OneDrive), INTG-023 (Nextcloud), INTG-024 (WebDAV), INTG-030 (object storage), then the rest.

## Acceptance Criteria

- [ ] A user with a fresh install connects Google Drive and Google Photos with only the browser sign-in.
- [ ] At least Dropbox, OneDrive and Nextcloud connect the same way.
- [ ] Every provider has docs in `docs/core/integrations.mdx` and the website.
