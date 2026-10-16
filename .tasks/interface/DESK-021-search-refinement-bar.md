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

- [ ] Pills for kind, extension, size, date and hidden refine the results
- [ ] Facet counts cover all results, not only the loaded page
- [ ] Hidden files are excluded unless the pill is on
- [ ] TypeScript types regenerated; core test covers the facets
