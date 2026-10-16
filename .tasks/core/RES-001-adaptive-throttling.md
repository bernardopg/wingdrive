---
id: RES-001
title: Adaptive Resource Throttling
status: To Do
assignee: jamiepine
parent: RES-000
priority: Medium
tags: [performance, mobile, core, upstream-backlog]
whitepaper: Section 7.1
milestone: M5
last_updated: 2026-10-16
---

## Description

Make background jobs like indexing and file transfers "good citizens" by implementing adaptive throttling. The system must monitor device status and automatically reduce its resource usage when the device is on battery power or under thermal pressure.

## Implementation Steps

1.  Integrate a cross-platform library to get real-time device status (power source, thermal state).
2.  Add hooks within the `JobExecutor` and `IndexerJob` to check this status.
3.  Implement logic to dynamically adjust resource usage, such as reducing the number of concurrent tasks or introducing delays between operations.

## Acceptance Criteria

- [ ] On battery power, background CPU usage is reduced by at least 50%.
- [ ] Indexing jobs are automatically paused or slowed when the system reports high thermal pressure.

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
