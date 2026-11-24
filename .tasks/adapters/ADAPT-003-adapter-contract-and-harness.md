---
id: ADAPT-003
title: "Shared adapter contract and test harness"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, testing, core]
related_tasks: [ADAPT-001]
last_updated: 2026-11-24
---

## Description

The eleven adapters were written against a prototype and have no tests. Before finishing any of
them, fix the contract they share and build the harness every adapter task uses: how config arrives,
how records, deletions and cursors are printed, how errors are reported, and how a test runs a real
script against fixtures.

## Implementation Steps

- [ ] Document the stdin config, JSONL record, delete, cursor and error lines in `docs/core/adapters.mdx`, matching `ScriptAdapter`
- [ ] Python helper module shared by adapters (`adapters/_lib/`) for config, emitting records, cursors and path defaults per platform
- [ ] Rust integration test helper that runs an adapter folder through `ScriptAdapter` with a fixture config and returns the stored records
- [ ] CI runs every adapter's `python3 -m unittest` and the Rust adapter tests

## Acceptance Criteria

- [ ] Contract doc merged and referenced from AGENTS.md
- [ ] Harness used by ADAPT-010 first, then the others
- [ ] CI job green with the adapter tests
