---
id: ADAPT-027
title: "Rebuild a source's search index when an adapter update adds search fields"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S08
tags: [adapters, archive, search]
depends_on: [ADAPT-004]
related_tasks: [ADAPT-012, ADAPT-026]
last_updated: 2026-11-28
---

## Description

When an adapter update adds a field to `[search] search_fields`, `SourceManager::open_with_migration`
(`crates/archive/src/source.rs`) adds the column but skips the `AddFtsColumn` action: the
`search_index` FTS5 table and its triggers keep the old columns, so the new field is stored and
never searched. Found 2026-11-28 with GitHub 1.1.0, which adds `comments` to its search fields: a
source created with 1.0.x keeps comments out of its search until the user adds the source again.

## Implementation Steps

- [ ] On `AddFtsColumn`, drop and recreate `search_index` and its triggers from the current schema, then fill it from the primary table (`INSERT INTO search_index(search_index) VALUES('rebuild')` for external content, or a full reinsert)
- [ ] Run it inside the sync's migration, once, and log how many rows were indexed
- [ ] Test: a source created with the old schema finds a record by the new field after the migration

## Acceptance Criteria

- [ ] A GitHub source created with 1.0.4 finds an issue by a word that only appears in its comments after updating to 1.1.0 and syncing once
- [ ] A migration that adds no search field leaves the index untouched
