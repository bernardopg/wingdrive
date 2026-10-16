---
id: FORK-008
title: Call zfs and zpool by absolute path
status: In Progress
assignee: bernardopg
parent: FORK-002
priority: Medium
milestone: M2
sprint: S03
tags: [fork, upstream, port, volumes, linux]
last_updated: 2026-10-16
---

## Description

Port of upstream `5f49a0141`, core part. A daemon started as a systemd user
service on Debian or Ubuntu has no `/usr/sbin` on its `PATH`, so
`core/src/volume/fs/zfs.rs` cannot find `zfs`/`zpool` and ZFS capacity
detection fails. Resolve the binaries from the usual locations before falling
back to `PATH`.

## Acceptance Criteria

- [ ] `zfs`/`zpool` are found when `PATH` lacks `/usr/sbin` and `/sbin`
- [ ] A unit test covers the lookup order
