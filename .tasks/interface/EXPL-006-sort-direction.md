---
id: EXPL-006
title: Ascending and Descending Sort in Directory Listings
status: In Progress
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

- [x] Core directory listing accepts an optional `sort_direction` (reuses search `SortDirection`) and applies it in SQL and in-memory sorting
- [x] TypeScript types regenerated, no hand-written backend types
- [x] List view header toggles direction on repeated click and shows a caret up/down
- [x] Sort menu flips direction when the active option is chosen again and shows an arrow
- [x] Grid, list, column, size, and knowledge views pass the same direction (media uses `files.media_listing`, which has its own ordering; out of scope)
- [x] Direction persists per tab with the other explorer state (TabManager)
- [x] Rust test covers default and explicit ascending/descending order
- [ ] Runtime check in the Linux desktop app (done together with TAURI-006)

## Notes

- Without `sort_direction` each key keeps its previous default (name/type ascending, modified/size descending), so existing clients are unchanged.
- The keyboard-navigation listing query omitted `folders_first`, so arrow keys could walk a different order than the one on screen. It now sends the same input as the visible listing.

## Implementation Files

- `core/src/ops/files/query/directory_listing.rs`
- `packages/interface/src/routes/explorer/` (views, context)
