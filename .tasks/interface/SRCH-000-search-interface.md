---
id: SRCH-000
title: Search Interface
status: In Progress
assignee: jamiepine
parent: UI-000
priority: Medium
tags: [search, interface, upstream-backlog]
whitepaper: Section 6.0
last_updated: 2026-10-16
milestone: M5
---

## Description

Current state: the Explorer top-bar search, debounce, backend query, and shared grid/list/media result rendering are implemented. Structured filters, operators, saved searches, history, semantic mode, and scale validation remain.

Build the search interface with support for full-text search, filters, and semantic search. Displays results in grid/list views with highlighting.

## Features

- Global search bar in top bar
- Real-time search as you type
- Filters (file type, date, size, tags, location)
- Search operators (AND, OR, NOT, quotes)
- Saved searches
- Search history
- Semantic search integration

## Implementation Notes

- Integrates with SEARCH-000 backend
- Uses async search jobs for large queries
- Debounced input for performance
- Results virtualized for large result sets
- Highlight matching terms

## Acceptance Criteria

- [x] Search bar in top bar
- [x] Real-time results as you type (debounced)
- [ ] Filter panel with common filters
- [ ] Search operators work correctly
- [ ] Save searches for reuse
- [ ] Search history dropdown
- [ ] Semantic search option (if enabled)
- [x] Results in grid/list view
- [ ] Click result to open/preview
- [ ] Performance with 100k+ indexed files

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
