---
id: EXPL-002
title: File List View with Sortable Columns
status: Done
assignee: jamiepine
parent: EXPL-000
priority: High
milestone: M1
sprint: S01
tags: [explorer, views, performance]
whitepaper: N/A
last_updated: 2026-10-02
---

## Description

Current state: list rendering, TanStack virtualization, sorting, selection, keyboard navigation, and file opening are implemented. Multi-column sorting and column resize passed runtime verification.

Implement a list view for files with sortable columns showing name, size, date modified, kind, and tags.

## Implementation Notes

- Use TanStack Table for column management
- TanStack Virtual for row virtualization
- Sortable columns with multi-column sort
- Resizable columns with drag handles
- Icon + text for file type

## Acceptance Criteria

- [x] List shows files with columns: Name, Size, Modified, Kind, Tags
- [x] Click column header to sort
- [x] Multi-column sort with Shift + click
- [x] Drag column edges to resize
- [x] Virtual scrolling for large lists
- [x] Selection works same as grid view
- [x] Keyboard navigation (up/down arrows)

## Verification (2026-10-02)

Production Playwright validates visible columns, numeric/date multi-sort with Shift, column resize, virtual rows and arrow selection.
