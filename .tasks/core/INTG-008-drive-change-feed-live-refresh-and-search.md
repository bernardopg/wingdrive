---
id: INTG-008
title: Drive change feed, live refresh and search
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Medium
milestone: M5
tags: [integrations, google, drive, sync, search]
last_updated: 2026-11-12
related_tasks: [CLOUD-004, FILE-003]
---

## Description

Changes made in Drive appear only after the 30-second listing cache expires and the user refreshes, and Drive files are not in library search because the location is not indexed.

## Implementation Steps

1. Use the Drive changes API (`changes.getStartPageToken`, `changes.list`) per account to invalidate listings and emit resource events.
2. Let cloud sync use the change feed instead of walking the whole folder each run.
3. Index Drive metadata (names, sizes, dates, MIME, MD5) without downloading content, so Drive files appear in search; validate the generic cloud indexer against Drive or add a Drive-specific one.

## Acceptance Criteria

- [ ] A file created in the Drive web UI shows up in an open Explorer tab without a manual refresh.
- [ ] Searching the library finds Drive files by name.
