---
id: NET-007
title: Paired devices show in the overview without a connection event
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [network, pairing, interface, verification]
last_updated: 2026-11-02
---

## Description

The overview `DevicePanel.tsx` asks for `show_paired: true`, so a device paired
earlier should be listed right after the app starts, before the peer connects.
The code came in with #231 (ported from upstream `9d948817e`); this task holds the
runtime check that the old port task (FORK-007, removed on 2026-11-02) left open.

## Acceptance Criteria

- [x] Overview device query passes `show_paired: true` (#231)
- [x] With a paired peer offline, the peer is listed in the overview after a restart

## Evidence

- 2026-11-02, packaged build: the app's daemon and a second isolated daemon paired with `wing network pair generate` / `join`. With both stopped, the app started on its own daemon and the overview showed "2 Devices" and the peer's card marked Offline, plus the peer in the sidebar Devices group.
