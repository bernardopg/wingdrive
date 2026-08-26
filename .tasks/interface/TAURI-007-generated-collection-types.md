---
id: TAURI-007
title: Generate Array Types for Collection Queries
status: To Do
assignee: unassigned
parent: TAURI-000
priority: Medium
tags: [tauri, typescript, codegen, bug]
last_updated: 2026-08-24
---

## Description

Generated outputs such as `sources.list` and `sources.list_items` currently use one-element tuple types (`[SourceInfo]`, `[SourceItem]`) for variable-length arrays. This forces incorrect narrowing in otherwise type-safe UI code.

## Acceptance Criteria

- [ ] Specta generation emits `SourceInfo[]` and `SourceItem[]`
- [ ] No UI casts are required for source collections
- [ ] Generated-type test covers zero, one, and multiple records
