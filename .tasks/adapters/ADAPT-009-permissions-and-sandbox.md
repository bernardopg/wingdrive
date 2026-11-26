---
id: ADAPT-009
title: "Adapter permissions and process sandbox"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, security, core]
related_tasks: [ADAPT-004, ADAPT-007]
last_updated: 2026-11-26
---

## Description

Adapters run third-party code on the user's machine and read private data. Each adapter declares what
it needs, the user sees it before install, and the daemon runs the adapter with only that.

## Implementation Steps

- [x] Permission model in the manifest: `read` paths (with platform defaults), `network` hosts, `secrets` keys, `tools` it may run (`{config.<key>}` and `~` expansion in `read`)
- [x] Linux: run the adapter under bubblewrap with read-only binds for declared paths, a private state directory, and no network unless hosts are declared; macOS and Windows: best-effort limits recorded in the docs (`crates/archive/src/adapter/sandbox.rs`; macOS and Windows unconfined, recorded in `docs/core/adapters.mdx` and shown in the store)
- [x] Minimal environment (no inherited tokens), CPU time and memory limits, kill on timeout (2 GiB RLIMIT_AS, RLIMIT_CPU = runtime.timeout, killed at runtime.timeout)
- [ ] Secrets passed per run from the secret store, never written to the source database or logs
- [ ] The Coverage workflow runs the confinement tests. Since 2026-11-25 its `Rust tests` job fails
      five `script_adapter_test` cases with `bwrap: loopback: Failed RTM_NEWADDR: Operation not
      permitted` (runs 45852694186 on `ce7f4ec`, and the push run on `66e0ce0c4`). `ci.yml` installs
      bubblewrap before its Rust gates and `coverage.yml` does not; check whether that is the
      difference or whether the runner blocks the network namespace, then either fix the runner
      or set `WINGDRIVE_ADAPTER_SANDBOX=off` for coverage only.

## Acceptance Criteria

- [x] Tests prove an adapter cannot read an undeclared path or reach the network without declaring it on Linux (`script_adapter_test.rs`: `adapters_see_only_declared_paths_and_write_only_their_state`, `adapters_without_hosts_have_no_network`, `adapters_stop_at_their_timeout_and_memory_limit`; real Obsidian (50 notes) and GitHub (289 repositories) syncs inside bwrap on 2026-11-25)
- [x] Store UI shows the permissions from the manifest (plus whether this system enforces them)
