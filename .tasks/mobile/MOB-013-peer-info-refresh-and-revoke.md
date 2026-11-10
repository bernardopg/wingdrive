---
id: MOB-013
title: "Refresh peer device info on reconnect and propagate unpairing"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S05
tags: [network, pairing, core]
last_updated: 2026-11-10
---

## Description

`DeviceInfo` (name, type, OS, app version) is sent once, inside the pairing request. A peer that
renames itself, upgrades or fixes its type keeps the old values on every paired device until it is
re-paired. Found on 2026-10-17: the Moto G7 renamed itself to "motorola moto g(7)" and reports
`Mobile`, while the desktop still shows "WingDrive Mobile" and `Desktop`. Revoking a device on one
side does not tell the other side, which keeps it listed as paired and connected.

## Acceptance Criteria

- [x] On connect, peers exchange current `DeviceInfo` and the registry updates the stored copy
- [x] `network.device.revoke` notifies the peer when it is reachable, and the peer drops the pairing
- [x] Test covering both paths

## Implementation

- `DeviceMessage::{Info, InfoAck, Unpaired, UnpairedAck}` in the messaging protocol
  (`core/src/service/network/protocol/device_messages.rs`). `PeerDeviceInfo` has only the fields
  the owner may change; identity comes from the authenticated connection.
- The registry notifies a channel on every transition to connected (`mark_connected`,
  `set_device_connected`, `update_device_from_connection`); `NetworkingService` answers each one
  by sending the local description. `update_peer_info` writes memory, slug cache and storage.
- `network.device.revoke` calls `notify_unpaired` (5 s wait) before `forget_paired_device`, closes
  the connections to that node and returns `peer_notified`.

## Evidence

- 2026-11-10: unit tests in `registry.rs`, `messaging.rs` and `device_messages.rs` (12). The
  messaging ones push framed messages through `handle_stream` over an in-memory stream: info and
  unpair from the paired node, both from an unknown node, and oversized info.
- Two isolated daemons on one machine (`--instance m13a`, `m13b`), paired with
  `wing network pair generate` and `join`. B was stopped, renamed to "Estudio B" with form factor
  `Server` in `device.json` and started again: A listed `Estudio B`, `Type: Server` about 16 s
  later, with `Device <id> now reports itself as Estudio B (Server, linux, 3.0.1)` in its log.
- `wing network revoke <B> -y` on A printed `The device dropped the pairing on its side.`; both
  `wing network devices` then answered `No paired devices`, and B stayed that way.

## Not covered

- A peer that is offline during the revocation keeps the pairing. Answering its next connection
  with "not paired" was left out: the daemon accepts connections before it loads its paired
  devices, so a restart could make a valid peer drop the pairing.
- Devices already registered in a library keep the name stored in the library database; that row
  belongs to library sync.
- No phone was used; the Moto G7 case is the same code path with `Mobile`.
