---
id: ADAPT-026
title: "Library search finds adapter source items"
status: Done
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, search, core, interface]
depends_on: [ADAPT-004]
related_tasks: [ADAPT-010, ADAPT-012, ADAPT-014, ADAPT-027, ADAPT-030]
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

- [x] A library query that searches every source of the library through `ArchiveEngine::search`, with the source, model, title, subtitle, date and preview of each hit, and a limit
      (`sources.search`: id, external ID, title, subtitle, date, a 280 character preview, a snippet
      with the matched terms marked, source ID and name, adapter ID, data type and a `flagged`
      flag; 50 hits by default, 200 at most. `sources.get_item` returns one item whole. The
      router now sorts hits best first (it sorted bm25 ranks worst first) and takes typed text,
      not FTS5 syntax.)
- [x] Only records the safety screening let into the index are returned
      (the index triggers already skip quarantined records; `fts_search` also filters on
      `_safety_verdict IN ('safe', 'flagged')`. Test `search_skips_quarantined_records`.)
- [x] The search screen shows source hits next to files, grouped by source, and opens the item in its source
      (Library scope only: the folder and location scopes are places on disk. A hit opens
      `/sources/<id>?q=<words>&item=<item id>` with the item open beside the list, and the source
      page's own search uses the index instead of filtering the loaded page.)
- [x] Every new UI text through i18n (`en` and `pt`)

## Acceptance Criteria

- [x] Searching a word from an Obsidian note and from a GitHub issue finds both in the library search, on an isolated daemon
      (2026-11-28, isolated `wing-server` built from this branch, synthetic data only: obsidian
      1.1.4, github 1.1.0 and coding-sessions 0.2.0 installed from store index; a 10 note vault,
      the GitHub adapter against a local fake API with user `ana` (8 items), 4 synthetic Claude
      Code and Codex sessions. `harness` found 7 hits: 2 GitHub, 3 vault, 2 sessions, next to 4
      files of an indexed folder. Screenshots in the PR and on wingdrive.org/docs/adapters.)
- [x] A source removed from the library no longer returns hits
      (same instance: after `sources.delete` of the sessions source, `harness` returned 5 hits from
      the two other sources. Tests `search_spans_sources_and_drops_deleted_ones` and
      `search_spans_sources_and_forgets_removed_ones`.)
- [x] Tests cover the query across two sources and an empty library
      (`wing-archive`: `search_spans_sources_and_drops_deleted_ones`, `search_skips_quarantined_records`,
      `search_orders_hits_best_match_first`, `fts::tests`; `wing-core`:
      `data::manager::tests::search_spans_sources_and_forgets_removed_ones`,
      `ops::sources::search::query::tests`; bun: `components/Sources/sourceSearch.test.ts`.)

Follow-ups are in ADAPT-030; rebuilding an index when search fields grow is ADAPT-027.
