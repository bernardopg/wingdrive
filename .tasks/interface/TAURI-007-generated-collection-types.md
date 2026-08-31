---
id: TAURI-007
title: Generate Array Types for Collection Queries
status: Done
assignee: unassigned
parent: TAURI-000
priority: Medium
tags: [tauri, typescript, codegen, bug]
last_updated: 2026-08-26
---

## Description

Generated outputs such as `sources.list` and `sources.list_items` currently use one-element tuple types (`[SourceInfo]`, `[SourceItem]`) for variable-length arrays. This forces incorrect narrowing in otherwise type-safe UI code.

## Acceptance Criteria

- [x] Specta generation emits `SourceInfo[]` and `SourceItem[]`
- [x] No UI casts are required for source collections
- [x] Generated-type test covers zero, one, and multiple records

## Implementation

- `extract_type_name` in `core/src/infra/wire/type_extraction.rs` formatted `Vec<T>` with Swift array syntax (`[T]`) and the TypeScript generator reused it, producing one-element tuples.
- Type references are now language-aware: `extract_type_name_swift` keeps `[T]`, `extract_type_name_typescript` emits `T[]`, both driven by a parser that matches the outermost generic constructor only (nested `Option` inside `Vec` previously corrupted the parse).
- The TypeScript generator consumes the new `*_name_ts` metadata fields; Swift output is unchanged.
- Regenerated types convert six collection outputs (`libraries.list`, `adapters.config`, `adapters.list`, `devices.list`, `sources.list`, `sources.list_items`) from tuples to arrays.
- Removed the `sourcesRaw?.slice()` tuple workaround in the Sources route.
- Rust unit tests cover both languages plus nested generics; a ts-client test pins the generated union to array outputs across zero, one, and many records and fails on any `[T]` output member.
