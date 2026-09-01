---
id: SAFE-005
title: "Add free space preflight to write-heavy jobs"
status: "Done"
assignee: unassigned
priority: "High"
tags: [core, jobs, safety]
last_updated: 2026-08-31
---

## Description

Copy, thumbnail, proxy and thumbstrip jobs wrote until the target volume ran
out of space. A copy that fills its destination leaves truncated files behind,
and derived-data jobs write thousands of small files into the library folder,
so exhausting that volume can degrade the whole system before any error
surfaces. Filesystems also misbehave before they reach zero: ext4 fragments in
the last few percent and btrfs can fail to allocate metadata block groups while
raw bytes remain.

## Acceptance Criteria

- [x] Copy jobs refuse when the destination cannot hold the estimated size
- [x] Derived-data jobs refuse to start on a volume without headroom
- [x] A reserve stays free rather than filling the volume exactly
- [x] Unverifiable mounts do not block valid writes
- [x] Unit tests cover fit, overflow, reserve and missing destinations

## Implementation

- `core/src/infra/fs/free_space.rs` resolves the longest matching mount point
  through `sysinfo`, so nested mounts answer for themselves. Destinations that
  do not exist yet resolve through their nearest existing ancestor.
- `ensure_space_for` adds a 256 MiB reserve on top of the requested size.
  `ensure_headroom` is the same check with no payload, used where no size
  estimate exists.
- `FileCopyJob::run` checks the resolved destination against
  `estimated_total_bytes` right after computing it, before any file is written.
- Thumbnail, proxy and thumbstrip jobs check headroom on the library folder as
  their first action.
- An unmatched mount returns `Ok`: refusing there would break valid writes to
  network and virtual filesystems that report no capacity.

## Validation

- 5 deterministic unit tests in `free_space.rs` cover nested-mount selection,
  unrelated path prefixes, unknown mounts, a write that fits, and the reserve
  boundary without depending on the host filesystem.
- `cargo test -p sd-core --lib` passes 356 tests.
