---
id: SEARCH-004
title: Indexed search misses substrings that folder search finds
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [search, indexing, bug]
last_updated: 2026-11-03
---

## Description

Seen on 2026-11-02 while taking README screenshots. In a folder that is not indexed,
searching `ar` in This Folder lists `harbor-lights.jpg`, `night-market.jpg` and
`wallpaper-dark.png` (ephemeral search matches substrings). After the folder is added as a
location, the same search returns 0 results: the persistent FTS search only matches word
prefixes. The result set should not change because a folder was indexed.

## Acceptance Criteria

- [x] A query matches the same names in indexed and unindexed folders
- [x] `core/tests/search_test.rs` covers a mid-word query on both paths

## Evidence

- Persistent search: after the FTS5 match, `append_substring_matches` adds entries whose full name
  (`name.extension`) contains the query (`LIKE '%q%'`, same folder scope, ranked after FTS hits).
- Ephemeral search returned only exact matches, or only prefix matches, when there were any, so `test` found
  `testcase.txt` but not `my_test_data.txt`. It now returns every name containing the query, ranked by
  `score_match` (exact, then prefix, then position).
- `core/tests/search_test.rs`: new `test_mid_word_query_matches_in_indexed_and_unindexed_folders` (`ar` finds
  `harbor-lights`, `night-market` and `wallpaper-dark` on both paths; it fails without the fix). Four tests
  in the file already failed on `main`: three relied on the query `a` matching names without an `a` and now
  use an empty Fast-mode query; the content-type filter test indexed in Shallow mode, which creates no
  content identity, and now uses Content mode. 14/14 pass.
- Desktop debug build, 2026-11-03: in location `loc`, searching `port` listed `annual-report-2026.txt` (daemon
  log: persistent search, 1 result); in an unindexed folder, `ile` listed its three `file*.txt`.
