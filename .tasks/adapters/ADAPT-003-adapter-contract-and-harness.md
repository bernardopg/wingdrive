---
id: ADAPT-003
title: "Adapter contract and test harness"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, testing, core]
related_tasks: [ADAPT-007, ADAPT-008]
last_updated: 2026-11-24
---

## Description

The prototype adapters were written against an older engine and have no tests. Fix the contract
every adapter follows and build the harness that the template, the adapters' own CI and the registry
bots run: how config arrives, how records, deletions and cursors are printed, how errors are
reported, and how a test runs a real adapter against fixtures.

## Implementation Steps

- [ ] Document the manifest (`adapter.toml`), the stdin config, JSONL record, delete, cursor and error lines in `docs/core/adapters.mdx`, matching `ScriptAdapter`
- [ ] Version the contract (`contract = 1` in the manifest) so the daemon can refuse adapters it does not understand
- [ ] The contract is a process speaking JSONL on stdin and stdout, so it covers both Python scripts and compiled adapters (Rust, Go or any language); the manifest says `runtime = "python3"` or `runtime = "binary"`
- [ ] Python helper package (`wingdrive-adapter-kit`) for config, emitting records, cursors, platform path defaults and secrets
- [ ] Harness that runs an adapter folder through `ScriptAdapter` with a fixture config and checks the stored records; usable from Rust tests and as a command (`wing adapter test`)
- [ ] The archive engine finds adapters only where the store installs them; drop the `CARGO_MANIFEST_DIR` lookups

## Acceptance Criteria

- [ ] Contract doc merged and referenced from AGENTS.md
- [ ] Harness passes on the template adapter and is used by ADAPT-007 and every adapter task
- [ ] CI job green with the harness tests
