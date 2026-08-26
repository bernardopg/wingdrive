---
id: TAURI-003
title: Fix Content Kind Filter Parity
status: Done
assignee: unassigned
parent: TAURI-000
priority: High
tags: [tauri, search, explorer, bug]
last_updated: 2026-08-24
---

## Description

`files.content_kind_stats` reported two images and one video from orphaned `content_identities` even though the audited library had no indexed file entries. Count navigable files and use the same canonical `content_identity.kind_id` in persistent search.

## Acceptance Criteria

- [x] The stats and filtered search use the same content-kind semantics
- [x] Each non-zero File Kinds card opens matching Explorer results
- [x] A regression test covers content with and without `content_identity`
- [x] Zero-count kinds remain non-interactive
