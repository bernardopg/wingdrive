---
id: ADAPT-027
title: "Source search follow-ups: prefixes, scopes and freshness"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
tags: [adapters, search, core, interface]
depends_on: [ADAPT-026]
related_tasks: [ADAPT-010, ADAPT-012, ADAPT-014]
last_updated: 2026-11-28
---

## Description

ADAPT-026 made the library search reach source items. Checking it on an isolated instance left
these gaps, none of which blocks the adapter tasks.

## Implementation Steps

- [ ] Prefixes find stemmed words while typing: source indexes use the `porter` tokenizer, which stems the prefix too, so `harn` does not find `harness` (stem `har`). Options: a second unstemmed FTS5 table or column for prefix queries, or the `trigram` tokenizer, with a migration that rebuilds existing indexes
- [ ] Source hits outside the Library scope: the Explorer starts searches in *This Folder*, where source hits do not show. Offer them there (a "N matches in your sources" link that switches to Library) or start search in Library when the folder has no match
- [ ] Results follow syncs: `sources.search` results are cached by query and do not refresh when a sync changes the source, until the query changes. Invalidate them on the source sync job's completion
- [ ] Hits ranked across sources on one scale: each source ranks with its own bm25 statistics, so a small source's hits can outrank better matches of a large one
- [ ] `sources.get_item` returns `Option<SourceItem>`, but the generated TypeScript type drops the `null` (as for every `Option` query output, such as `files.by_id`); fix the generator so callers see `SourceItem | null`
- [ ] The mobile app shows source hits in its search

## Acceptance Criteria

- [ ] Typing `harn` finds an item that contains `harness`
- [ ] A search started in a folder tells you about matches in your sources
- [ ] An item a sync adds shows in a search already on screen
