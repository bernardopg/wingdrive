---
id: TAURI-002
title: Implement Persistent Favorites
status: Done
assignee: unassigned
parent: TAURI-000
priority: High
tags: [tauri, explorer, favorites]
last_updated: 2026-08-24
---

## Description

Replace the Favorites placeholder with a backend-backed collection. Favorite state must come from generated `File.favorite` data and persist across restart and sync.

## Acceptance Criteria

- [x] Backend action adds and removes a favorite
- [x] Resource events update every subscribed window
- [x] Inspector action reflects persisted state
- [x] Favorites route lists the same records after restart
- [x] Empty and error states are explicit

## Validation

- Typecheck, `sd-core` check, Tauri build and task validation pass.
- SQLite tests cover persisted favorite filtering.
- The normalized-query test covers invalidation when a file changes favorite state.
- A disposable indexed location proved add, remove, filtered listing and persistence after daemon restart.
- Two independent TCP clients received the same `favorite=false` ResourceEvent. The location and fixture were then removed, returning the library to zero entries and zero favorites.
