---
id: INTG-007
title: Shared drives and "Shared with me" in Google Drive
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
tags: [integrations, google, drive]
last_updated: 2026-11-12
related_tasks: [CLOUD-004]
---

## Description

`GoogleDriveBackend` only browses My Drive. Many users keep work files in shared drives or in folders shared with them.

## Implementation Steps

1. List shared drives (`drives.list`) and expose each as a top folder, or as its own volume.
2. Add a virtual "Shared with me" folder (`sharedWithMe = true`), read-only for items the user cannot edit.
3. Respect `capabilities` (canEdit, canDelete, canAddChildren) in the backend and the UI.

## Acceptance Criteria

- [ ] Shared drives and shared items browse and copy like My Drive.
- [ ] Actions the user cannot perform are disabled, not failing.
