---
id: EXPL-006
title: Ascending and Descending Sort in Directory Listings
status: To Do
assignee: bernardopg
parent: EXPL-000
priority: High
milestone: M1
sprint: S01
tags: [explorer, sorting, core]
last_updated: 2026-10-01
---

## Description

`DirectoryListingInput` accepts `sort_by` (Name, Modified, Size, Type) but no direction, so every column sorts one way only. Every desktop file manager toggles direction by clicking the column header again.

## Acceptance Criteria

- [ ] Core directory listing accepts a sort direction and applies it
- [ ] TypeScript types regenerated, no hand-written backend types
- [ ] List view header toggles direction on repeated click and shows an indicator
- [ ] Grid, media, and column views honor the same direction
- [ ] Direction persists with the existing per-location view preferences
- [ ] Rust test covers ascending and descending order

## Implementation Files

- `core/src/ops/files/query/directory_listing.rs`
- `packages/interface/src/routes/explorer/` (views, context)
