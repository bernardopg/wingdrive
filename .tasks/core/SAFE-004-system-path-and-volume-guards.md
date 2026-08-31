---
id: SAFE-004
title: "Protect system paths and privileged volume operations"
status: "Done"
assignee: "jamiepine"
priority: "High"
tags: [core, filesystem, safety, volumes, locations]
---

## Description

Indexing operating-system trees can create unbounded watcher and database load. Ejecting a root or system volume can destabilize the running machine. Privileged volume tests must also require explicit operator intent before they format or mount devices.

## Implementation Steps

- [x] Reject filesystem roots, Linux system trees, and WingDrive's data directory as locations
- [x] Reject remote physical paths in the local location action
- [x] Refuse eject requests for system, root, and `/home` mounts
- [x] Ignore privileged volume tests by default and require an environment gate
- [x] Verify focused tests and workspace compilation

## Acceptance Criteria

- `/`, `/proc`, `/sys`, `/dev`, `/run`, `/boot`, and the active data directory cannot be added as locations
- User data and removable media paths remain valid
- System roots and `/home` cannot be ejected through the volume action
- Destructive volume tests require both `--ignored` and `WINGDRIVE_PRIVILEGED_VOLUME_TESTS=1`
