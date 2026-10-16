---
id: DESK-021
title: Search refinement bar (kind, extension, size, date, hidden)
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, search, interface, core]
last_updated: 2026-10-16
---

## Description

Re-implementation of upstream `81accc3ad`, `3f4b67100` and `7b6f8ac9f`.
`SearchToolbar.tsx` filters by kind only. Add refinement pills for kind,
extension, size, modified date and hidden files, with facet counts computed
over the whole result set in `ephemeral_search.rs`.

## Acceptance Criteria

- [x] Pills for kind, extension, size, date and hidden refine the results
- [x] Facet counts cover all results, not only the loaded page
- [x] Hidden files are excluded unless the pill is on
- [x] TypeScript types regenerated; core test covers the facets
- [ ] Verified on the Linux desktop build

## Evidence

- Core: `SearchFacets` gains `kinds` (wire spelling of `ContentKind`) and an `add` fold; searches
  return a `SearchPage` (page, total before paging, facets over every filtered match).
- Ephemeral search (`ephemeral_search.rs`) drops hidden entries when `include_hidden` is `false`,
  folds facets before paging, and now honours `offset` (page size still capped at 200).
- Persistent FTS search (`query.rs`) applied only kind, favorite and tag filters, after SQL paging.
  It now reads up to `FTS_MATCH_CAP` (5000) matches, applies every entry filter (extension, size,
  date, kind, favorite, hidden, redundancy) in one query that also feeds the facets, then pages.
  `total_found` is the filtered match count instead of the old `LIKE` recount.
- Hidden means a dot-prefixed name, the same rule `files.directory_listing` uses
  (`filters::not_hidden_condition`).
- UI: `SearchToolbar.tsx` filter panel has kind, extension (top 8 from facets), size, modified
  (past day/week/month/year) and hidden rows with counts, plus the true result count. The hidden pill
  defaults to the Show Hidden Files view setting. Strings in `en` and `pt`.
- Tests: `test_ephemeral_search_facets_and_hidden_filter` and
  `test_persistent_search_facets_and_hidden_filter` in `core/tests/search_test.rs`; extra cases in
  `searchQuery.test.ts`.

## Follow-ups

- Runtime check on the Linux desktop build.
- Matches past the 5000 FTS cap are not reachable; move paging into SQL if libraries outgrow it.
- Counts of the active dimension are hidden (facets are computed after all filters); per-dimension
  disjunctive facets would need one facet pass per dimension.
- Upstream rebuilt the bar from `@wingdrive/primitives`; this keeps the existing toolbar buttons.
- Four older tests in `core/tests/search_test.rs` already fail at `88e558800` (ephemeral substring,
  date and filter tests, persistent filter test). They fail the same way with this change.
