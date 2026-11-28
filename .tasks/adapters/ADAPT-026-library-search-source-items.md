---
id: ADAPT-026
title: "Library search finds adapter source items"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
tags: [adapters, search, core, interface]
depends_on: [ADAPT-004]
related_tasks: [ADAPT-010, ADAPT-012, ADAPT-014]
last_updated: 2026-11-28
---

## Description

Every adapter task asks that its items "are found by the library search", and none can be: the
library search (`search.files`) queries only the file index. Each source already keeps an FTS5
index over its `[search] search_fields`, and `ArchiveEngine::search` searches across sources, but
nothing in the core calls it. Checked 2026-11-28 on an isolated 3.8.2 instance: the GitHub source
matched `harness` in 51 items through its own index, while the library search had no way to reach
them.

## Implementation Steps

- [ ] A library query that searches every source of the library through `ArchiveEngine::search`, with the source, model, title, subtitle, date and preview of each hit, and a limit
- [ ] Only records the safety screening let into the index are returned
- [ ] The search screen shows source hits next to files, grouped by source, and opens the item in its source
- [ ] Every new UI text through i18n (`en` and `pt`)

## Acceptance Criteria

- [ ] Searching a word from an Obsidian note and from a GitHub issue finds both in the library search, on an isolated daemon
- [ ] A source removed from the library no longer returns hits
- [ ] Tests cover the query across two sources and an empty library
