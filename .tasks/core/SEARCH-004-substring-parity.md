---
id: SEARCH-004
title: Indexed search misses substrings that folder search finds
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [search, indexing, bug]
last_updated: 2026-11-02
---

## Description

Seen on 2026-11-02 while taking README screenshots. In a folder that is not indexed,
searching `ar` in This Folder lists `harbor-lights.jpg`, `night-market.jpg` and
`wallpaper-dark.png` (ephemeral search matches substrings). After the folder is added as a
location, the same search returns 0 results: the persistent FTS search only matches word
prefixes. The result set should not change because a folder was indexed.

## Acceptance Criteria

- [ ] A query matches the same names in indexed and unindexed folders
- [ ] `core/tests/search_test.rs` covers a mid-word query on both paths
