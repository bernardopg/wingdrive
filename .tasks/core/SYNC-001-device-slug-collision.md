---
id: SYNC-001
title: "Fix device slug collision aborting shared-change apply"
status: "Done"
assignee: unassigned
priority: "High"
tags: [core, sync, bug]
last_updated: 2026-08-31
---

## Description

Applying a shared device record accepted the incoming slug verbatim whenever the
device already existed locally. The `devices.slug` column is UNIQUE, so a slug
already held by a different row aborted the apply with
`UNIQUE constraint failed: devices.slug`. Two devices provisioned from machines
with the same hostname collide immediately, which is the common case for test
harnesses and for a user running two instances on one machine.

The failure was visible in every `sync_backfill_race_test` run: the device
snapshot apply errored, and the peer continued with incomplete shared state.

## Acceptance Criteria

- [x] Incoming slugs held by another device are renamed instead of failing
- [x] Re-applying a device's own slug does not rename it
- [x] Regression test covers collision and idempotency
- [x] `UNIQUE constraint failed: devices.slug` no longer appears in sync runs

## Implementation

- `Model::apply_shared_change` in `core/src/infra/db/entities/device.rs` now
  collects slugs from every *other* device (`Column::Uuid.ne(uuid)`) and passes
  them through `Library::ensure_unique_slug` on both the insert and the update
  path. Collision avoidance previously ran only on insert.
- `ensure_unique_slug` returns the input untouched when there is no conflict, so
  repeated applies of the same record are stable.

## Validation

- Two unit tests against an in-memory SQLite schema: collision renames to
  `archlinux-2` while the holder keeps `archlinux`; three consecutive applies of
  the same record leave the slug unchanged.
- `cargo test -p sd-core --test sync_backfill_race_test` passes 4/4 with zero
  occurrences of the previous UNIQUE constraint warning.
