---
id: VOL-001
title: Volume PhysicalClass and Location LogicalClass
status: To Do
assignee: jamiepine
parent: VOL-000
priority: High
tags: [volume, storage-tiering, classification, upstream-backlog]
whitepaper: Section 4.8
milestone: M5
last_updated: 2026-10-16
---

## Description

Implement the `PhysicalClass` for Volumes and `LogicalClass` for Locations. This is the foundation of the intelligent storage tiering system, allowing WingDrive to understand the physical characteristics of storage devices and the user's intent for the data stored on them.

## Implementation Steps

1.  Define the `PhysicalClass` enum (e.g., `SSD`, `HDD`, `Cloud`) for Volumes.
2.  Define the `LogicalClass` enum (e.g., `Hot`, `Warm`, `Cold`) for Locations.
3.  Implement the logic to associate a `PhysicalClass` with each `Volume`.
4.  Implement the logic to allow users to assign a `LogicalClass` to each `Location`.

## Acceptance Criteria

- [ ] The `PhysicalClass` and `LogicalClass` enums are defined.
- [ ] The system can correctly identify the `PhysicalClass` of a Volume.
- [ ] A user can assign a `LogicalClass` to a Location.

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
