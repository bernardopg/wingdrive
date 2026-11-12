---
id: INTG-009
title: "Google Photos: choose albums and show upload progress"
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
tags: [integrations, google, photos, ux]
last_updated: 2026-11-12
related_tasks: [CLOUD-004]
---

## Description

The context menu uploads without an album and reports only start and end. The backend already supports albums (`integrations.google.photos.albums`, `new_album_title`).

## Implementation Steps

1. Offer "New album..." and the existing WingDrive albums in the upload submenu.
2. Show per-file progress in the job panel.
3. Let the import pick a destination with the folder picker in one step.

## Acceptance Criteria

- [ ] Uploading into a new or existing album works from the Explorer.
- [ ] Long uploads show progress.
