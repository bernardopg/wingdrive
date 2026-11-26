---
id: CORE-026
title: Library lock left by an earlier run with the same PID blocks the library
status: In Progress
assignee: bernardopg
parent: CORE-000
priority: Medium
milestone: M5
sprint: S06
tags: [core, library, server, docker, bug]
related_tasks: [REL-004]
last_updated: 2026-11-26
---

## Description

`LibraryLock` treats a lock as stale when its PID is no longer running. In a container the server
is PID 1 on every run, so after a restart the lock the previous run left names a running process,
the new run itself, and the library fails to load with `AlreadyInUse`. The author's homelab works
around it with an entrypoint that deletes `*.winglibrary.lock` before starting.

## Implementation

The lock now stores a random ID of the process run that took it (`instance_id`). A lock with our
own PID and another run's ID, or none (written by an older binary), is stale. A second lock in the
same run still fails with `AlreadyInUse`.

## Implementation Steps

- [x] Store and compare the run ID.
- [x] Unit test `lock_from_an_earlier_run_with_our_pid_is_stale`.
- [ ] Homelab: drop the lock-deleting entrypoint once a release ships this.

## Acceptance Criteria

- [x] A restarted container loads its library without deleting the lock by hand.
- [x] Two opens of one library in one process still conflict.
