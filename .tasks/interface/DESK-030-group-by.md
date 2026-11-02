---
id: DESK-030
title: Group items by kind, date or size
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer]
sprint: S04
last_updated: 2026-11-02
---

## Description

Dolphin ("Show in Groups"), Finder ("Use Groups") and Windows Explorer ("Group by") split a folder
into labelled sections by the sort key. WingDrive only sorts.
## Implementation Steps

1. Derive the group key from the active sort: kind (content kind), date (Today, Yesterday,
   This week, This month, then by year), size (buckets from tiny to huge), name (first letter).
2. List view adds header rows to its virtualized rows; grid view builds rows per group so a
   header always starts a new row.
3. Keyboard navigation and range selection keep using the flat file order, and the toggle lives
   in the view settings, persisted per location like the sort.

## Acceptance Criteria

- [x] Grid and list views can group by the active sort key with section headers
- [x] Verified on the Linux desktop build

## Evidence

- `groupFiles`, `dateLabel`, `buildGridLayout`, `gridNeighbor` and `buildListLayout` in
  `packages/interface/src/routes/explorer/views/grouping.ts`, covered by `grouping.test.ts` in CI.
- "Show in Groups" (`groupItems`) is a per-tab setting next to "Folders First". `GridView` and
  `ListView` publish the regrouped order with `setCurrentFiles`, so Shift ranges follow the screen;
  `GroupHeader` draws the section title and count.
- Open: verification on the installed Linux package as part of DESK-017.

- Runtime, 2026-11-02: isolated Linux Tauri debug build with more than 100 files and several size/date buckets. Grid and list showed groups, ascending and descending name order, and Shift+Down selected across group boundaries. Type, date and size grouping were checked in grid, including reversed date order.
- Fixed name, date, size and type section order when folders-first and the selected sort direction disagreed. Fixed Shift+arrow range selection in the grid. `grouping.test.ts` covers the section order and virtualized row navigation.
