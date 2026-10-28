---
id: DESK-030
title: Group items by kind, date or size
status: To Do
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

Next up: third easiest open M2 item. Effort: one to two days, frontend only.

1. Derive the group key from the active sort: kind (content kind), date (Today, Yesterday,
   This week, This month, then by year), size (buckets from tiny to huge), name (first letter).
2. List view adds header rows to its virtualized rows; grid view builds rows per group so a
   header always starts a new row.
3. Keyboard navigation and range selection keep using the flat file order, and the toggle lives
   in the view settings, persisted per location like the sort.

## Acceptance Criteria

- [ ] Grid and list views can group by the active sort key with section headers
