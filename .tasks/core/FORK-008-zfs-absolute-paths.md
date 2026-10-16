---
id: FORK-008
title: Call zfs and zpool by absolute path
status: Done
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

- [x] `zfs`/`zpool` are found when `PATH` lacks `/usr/sbin` and `/sbin`
- [x] A unit test covers the lookup order

## Progress

- 2026-10-16: `resolve_bin` looks in `/sbin`, `/usr/sbin`, `/usr/local/sbin`
  once per binary and falls back to `PATH`; all four `zfs`/`zpool` calls use
  it. Test `resolve_bin_prefers_the_first_sbin_dir_that_has_it`.

