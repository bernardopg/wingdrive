---
id: LSYNC-024
title: Full reconciliation of device-owned data between peers
status: Done
assignee: bernardopg
parent: LSYNC-000
priority: High
milestone: M4
sprint: S06
tags: [core, sync, bug]
related_tasks: [LSYNC-020, CORE-023]
last_updated: 2026-11-26
---

## Description

The 3.8.1 sync fixes (#551) stopped new deletions and changes from getting lost, but what was lost
before them stayed lost. On 2026-11-26 the laptop and the homelab still disagreed: the homelab held
laptop entries the laptop had deleted (`IMG_7188` to `IMG_7191`, `zz-probe-d1`, the folder
`probeD`) and lacked some it had created.

Nothing in sync could repair that:

- An incremental catch-up asks for records and tombstones newer than the per-resource watermark.
  A deletion whose tombstone sits behind the watermark, or that never got a tombstone, is never
  fetched again.
- A full pass (`since = None`) returns records only: `StateRequest` sends no tombstones without
  `since`, and the receiver never removed anything it did not get.
- The "watermark too old, forcing full sync" path did not even fetch everything for device-owned
  models: `backfill_device_owned_state` ignored its `since` argument and used the per-resource
  watermarks.

## Implementation

- `catch_up_since` takes `reconcile`. With it, every device-owned model is fetched without a
  watermark, and for `location` and `entry` the receiver keeps the set of UUIDs the peer sent.
  After the last page it lists its own copies of that peer's records (the same ownership query the
  peer answered) and removes each one the peer did not send and that was last written here before
  the pass started, through the model's deletion (an entry takes its subtree). The time check keeps
  a record that arrived in real time during the pass.
- Volumes stay out: their sync timestamp is the owner's `last_seen_at`, not a local write time.
- Shared resources are requested without an HLC watermark in the same pass, so the peer adds its
  full shared state. Found on the homelab: `IMG_7188` to `IMG_7191` never applied because their
  content identities were created by the watcher before #551 and never entered the laptop's peer
  log, so no incremental catch-up could fetch them (`pending dependencies ... failed=5`).
- A reconciliation runs on the first catch-up with each peer, whenever the state watermark is
  stale, and then every `reconcile_interval_days` (default 7, 1 in the aggressive preset). The
  time of the last one is kept per peer in `device_state_reconciliations` in `sync.db`.
- `BackfillManager::reconcile_with_peer` runs one on demand.

## Implementation Steps

- [x] Reconcile `location` and `entry` after a full pass, removing stale local copies.
- [x] Schedule it per peer and record when it last ran.
- [x] Integration test: Bob drops Alice's folder and its child that Alice removed without a
      tombstone, keeps her other entries and gets back an entry and content identity he never
      received (`sync_backfill_test`).
- [x] Fetch the full shared state during a reconciliation.
- [x] Runtime: laptop, homelab and phone converge on one entry set after upgrading (2026-11-26,
      see below).

## Acceptance Criteria

- [x] A record the owner no longer has is removed from every peer by the next reconciliation.
- [x] A record the owner created during the pass is not removed.
- [x] The laptop and homelab residue from before 3.8.1 is gone after the upgrade.

## Runtime check (2026-11-26)

Branch builds on the author's laptop (daemon), homelab (`wing-server` image) and Moto G7
(`com.wingdrive.app.review`). Entries compared per owner, by UUID and name, against the laptop:

| Owner | Before | After |
|---|---|---|
| Laptop | homelab: 2 deleted entries kept (`l2`, `zz-probe-d1`), 4 created ones missing (`IMG_7188` to `IMG_7191`), 1 rename missing | identical on homelab and phone (2721) |
| Homelab | laptop: 10 deleted entries kept, 1 missing, 5 renames missing; phone: 1 missing | identical on laptop and phone (428) |
| Phone (`.review`) | identical | identical (59) |

The laptop removed 10 records in its first reconciliation with the homelab and the homelab 2 in
its first with the laptop. The 4 photos needed the full shared state: their content identities
never entered the laptop's peer log.

The release app on the same phone never holds the `.review` app's entries: the two never run at
the same time (only the app in the foreground runs its core) and peers only send their own data.
