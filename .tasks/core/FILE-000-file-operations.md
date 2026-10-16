---
id: FILE-000
title: "Epic: File Operations"
status: In Progress
assignee: jamiepine
priority: High
tags: [epic, core, file-ops, upstream-backlog]
whitepaper: Section 4
milestone: M5
last_updated: 2026-10-16
---

## Description

Encompasses all core file manipulation tasks, including copying, moving, deleting, and validation. These operations will be implemented as durable jobs orchestrated by the Action System.

## Child Tasks

- **FILE-001**: File Copy Job - Done
- **FILE-002**: File Deletion Job - Done
- **FILE-003**: Cloud Volume File Operations - Done
- **FILE-004**: Rename and Folders - Done
- **FILE-005**: Bidirectional Remote Copy - Done
- **FILE-006**: Path Intersection & Smart Diff - To Do

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
