---
id: MOB-013
title: "Refresh peer device info on reconnect and propagate unpairing"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
tags: [network, pairing, core]
last_updated: 2026-10-17
---

## Description

`DeviceInfo` (name, type, OS, app version) is sent once, inside the pairing request. A peer that
renames itself, upgrades or fixes its type keeps the old values on every paired device until it is
re-paired. Found on 2026-10-17: the Moto G7 renamed itself to "motorola moto g(7)" and reports
`Mobile`, while the desktop still shows "WingDrive Mobile" and `Desktop`. Revoking a device on one
side does not tell the other side, which keeps it listed as paired and connected.

## Acceptance Criteria

- [ ] On connect, peers exchange current `DeviceInfo` and the registry updates the stored copy
- [ ] `network.device.revoke` notifies the peer when it is reachable, and the peer drops the pairing
- [ ] Test covering both paths
