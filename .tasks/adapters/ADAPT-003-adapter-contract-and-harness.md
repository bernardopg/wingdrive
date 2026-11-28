---
id: ADAPT-003
title: "Adapter contract and test harness"
status: Done
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, testing, core]
related_tasks: [ADAPT-007, ADAPT-008]
last_updated: 2026-11-28
---

## Description

The prototype adapters were written against an older engine and have no tests. Fix the contract
every adapter follows and build the harness that the template, the adapters' own CI and the registry
bots run: how config arrives, how records, deletions and cursors are printed, how errors are
reported, and how a test runs a real adapter against fixtures.

## Implementation Steps

- [x] Document the manifest (`adapter.toml`), the stdin config, JSONL record, delete, cursor and error lines in `docs/core/adapters.mdx`, matching `ScriptAdapter`
- [x] Version the contract (`contract = 1` in the manifest) so the daemon can refuse adapters it does not understand
- [x] The contract is a process speaking JSONL on stdin and stdout, so it covers both Python scripts and compiled adapters (Rust, Go or any language); the manifest says `runtime = "python3"` or `runtime = "binary"`
- [x] Python helper package (`wingdrive-adapter-kit`) for config, emitting records, cursors, platform path defaults and secrets
      Merged as bernardopg/wingdrive-adapters#9 (2026-11-28). It is one file adapters copy into their
      folder, `kit/python/wingdrive_adapter.py`, not a PyPI package, because the daemon runs
      adapters with `python3 -I` and only the standard library. It covers config fields, secrets
      (replaced by `[secret]` in logs and errors), versioned cursors, contract lines,
      `config_home()`/`data_home()` per platform and `run`. `wingreg` accepts a vendored copy only
      when it matches `kit/python/releases.json`. The new `templates/python` ("text notes") uses it
      and passes `wing adapter test` (2 of 2 cases) with this branch's CLI.
- [x] Harness that runs an adapter folder through `ScriptAdapter` with a fixture config and checks the stored records; usable from Rust tests and as a command (`wing adapter test`)
      (`wing_archive::harness`, 2026-11-28). Cases live in `fixtures/<case>/` with `config.json`
      (`{fixture}` is the case folder) and `expected.json` (`counts`, `records` with fields,
      `absent`, `error`, `runs`, `idempotent`). Each case syncs into a temporary source, twice by
      default with the cursor passed on, and the second run must keep the record counts.
      `SourceDb::records` reads a model's fields for the comparison. The fixture adapter
      `crates/archive/tests/fixtures/harness/notes` has a passing case and an expected error;
      `harness_test.rs` also breaks copies of it (new ids per run, wrong fields, no cases) and
      checks the failures. `wing adapter test [dir] [--case name]` prints one line per case and
      exits 1 when a case fails; run on the laptop against the fixture (2 of 2 passed) and the
      official Obsidian adapter ("has no test cases"). `scripts/ci-rust.sh` runs the harness
      tests on Linux in the full scope. Docs: "Testing" in `docs/core/adapters.mdx`.
- [x] The archive engine finds adapters only where the store installs them; drop the `CARGO_MANIFEST_DIR` lookups
- [x] Contract 1 stdin carries `config`, the last `cursor` and `data_dir`; the daemon used to store cursors and never send them back
- [x] Deleting a linked or unscreened record works (junction rows first, FTS delete trigger only for indexed rows, old trigger repaired on open)
- [x] A failed run reports the adapter's last stderr line

## Acceptance Criteria

- [x] Contract doc merged and referenced from AGENTS.md (`docs/core/adapters.mdx`, AGENTS.md "Adapters (store)")
- [x] Harness passes on the template adapter and is used by ADAPT-007 and every adapter task
      (2026-11-28: `templates/python` passes `wing adapter test`, 2 of 2 cases. The registry's
      Validate workflow builds `wing-adapter-test` (#584) and runs it on every adapter, and an
      adapter without a `fixtures/<case>/expected.json` fails review (bernardopg/wingdrive-adapters#13).
      Run 46165091592: coding-sessions 2 of 2, github 5 of 5, obsidian 2 of 2, the template in
      `tests/test_submissions.py`.)
- [x] CI job green with the harness tests (#563: "Test the adapter contract harness" in Rust Workspace, run 46156548622, `harness_test` 4 passed; merged as 0138ac9)
