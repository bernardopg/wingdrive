---
id: FSYNC-006
title: "S07 carryover: validate CLI fetch flow across two isolated profiles"
status: To Do
assignee: bernardopg
parent: FSYNC-000
priority: High
milestone: M4
sprint: S08
tags: [sync, cli, e2e, validation, carryover]
related_tasks: [FSYNC-003, FSYNC-004, CORE-023]
last_updated: 2026-11-23
---

## Description

S07 step 4 requires evidence for "consulta e obtenção" via CLI in two independent profiles: search with source offline, reconnect, copy, verify hash, repeat with zero content bytes. `wing sync fetch` exists and has unit/process coverage, but the full two-daemon offline/online E2E flow was not executed with recorded evidence in this session.

## Acceptance Criteria

- [ ] Two isolated daemons/profiles: index file on A, query from B while A offline.
- [ ] Reconnect A, run fetch/copy, verify destination hash matches source.
- [ ] Repeat fetch reports already verified / zero content bytes transferred.
- [ ] Record commands, versions, bytes, and results in task evidence.
