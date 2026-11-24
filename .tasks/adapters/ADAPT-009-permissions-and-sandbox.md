---
id: ADAPT-009
title: "Adapter permissions and process sandbox"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, security, core]
related_tasks: [ADAPT-004, ADAPT-007]
last_updated: 2026-11-24
---

## Description

Adapters run third-party code on the user's machine and read private data. Each adapter declares what
it needs, the user sees it before install, and the daemon runs the adapter with only that.

## Implementation Steps

- [ ] Permission model in the manifest: `read` paths (with platform defaults), `network` hosts, `secrets` keys, `tools` it may run
- [ ] Linux: run the adapter under bubblewrap with read-only binds for declared paths, a private state directory, and no network unless hosts are declared; macOS and Windows: best-effort limits recorded in the docs
- [ ] Minimal environment (no inherited tokens), CPU time and memory limits, kill on timeout
- [ ] Secrets passed per run from the secret store, never written to the source database or logs

## Acceptance Criteria

- [ ] Tests prove an adapter cannot read an undeclared path or reach the network without declaring it on Linux
- [ ] Store UI shows the permissions from the manifest
