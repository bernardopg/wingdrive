---
id: CORE-023
title: A peer with no device-owned data forces a full sync every few seconds
status: In Progress
assignee: bernardopg
parent: LSYNC-000
priority: High
milestone: M4
sprint: S06
tags: [core, sync, performance, bug]
related_tasks: [CORE-022, BRAND-003, MOB-013]
last_updated: 2026-11-16
---

## Description

Found on 2026-11-15 while checking BRAND-003: a desktop daemon built from `main` shared a new library
with the moto g(7) on `v3.3.1`. The phone has no locations or entries, so it owns no device data.
In five minutes the desktop logged 56 times:

```
WARN wing_core::service::sync::backfill: State watermark is 106751991167 days old (> 25 days), forcing full sync to ensure consistency
```

about every 5 seconds, each with a `SyncError` event in the sync event log. The phone logged
`WARN ... entities::entry: No entries found for device, returning empty` at the same pace.

`catch_up_from_peer` (`core/src/service/sync/backfill.rs`) treats a missing state watermark as
`Duration::max_value()`, so a peer that never sent device-owned data is always "too old". Every
catch-up then runs as a full sync, and nothing ever sets the watermark, so the next catch-up does the
same. The age in the message is the sentinel, not a real age.

## Implementation Steps

- [x] A missing state watermark means "nothing received yet": the catch-up starts from the
      beginning, logs it at INFO and writes no SyncError event (`catch_up_from_peer`).
- [x] Find what triggers a catch-up every 5 seconds with an idle peer, and stop it while the peer is
      Ready and nothing changed.
- [x] "No entries found for device" is a normal answer for a device without locations: DEBUG.
- [x] A peer with no device-owned watermarks but a recent successful catch-up is skipped by the sync
      loop, instead of being treated as a new peer on every pass (`run_sync_loop`).
- [ ] Regression test with two in-process cores where one side owns no device data: one full sync,
      then incremental catch-ups only.

## Acceptance Criteria

- [ ] With the same setup (desktop and phone, the phone without locations), 10 idle minutes log no
      forced full sync after the first one and no WARN about watermarks.

## Cause (2026-11-16)

The sync loop (`run_sync_loop` in `core/src/service/sync/mod.rs`) runs every few seconds and
decides per connected peer from its watermarks in `sync.db`. A peer with no watermark is "new" and
gets a catch-up; one whose oldest watermark is older than 60 s is "stale" and gets one too. A phone
without locations sends no device-owned data, so it never gets a watermark and stayed "new" on every
pass, and a quiet peer's watermark only ages. `catch_up_from_peer` then read the missing watermark
as `Duration::max_value()`, hence "106751991167 days old" and a forced full sync each time.

The loop now remembers the last successful catch-up per peer and skips the peer for 60 s after it
(`CATCH_UP_INTERVAL`), whatever its watermarks say. After that window, a peer with no device-owned
watermarks is skipped when it has already been caught up, because the empty delta will not produce a
watermark; real-time broadcasts carry later changes and catch-up only repairs gaps.
