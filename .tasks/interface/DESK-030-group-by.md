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
last_updated: 2026-10-28
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
- [ ] Verified on the Linux desktop build

## Evidence

- `groupFiles`, `dateLabel`, `buildGridLayout`, `gridNeighbor` and `buildListLayout` in
  `packages/interface/src/routes/explorer/views/grouping.ts`, covered by `grouping.test.ts` in CI.
- "Show in Groups" (`groupItems`) is a per-tab setting next to "Folders First". `GridView` and
  `ListView` publish the regrouped order with `setCurrentFiles`, so Shift ranges follow the screen;
  `GroupHeader` draws the section title and count.
- Open: runtime check on the Linux desktop build (each sort key, Up and Down across sections,
  Shift range across sections, long folders scrolled).
