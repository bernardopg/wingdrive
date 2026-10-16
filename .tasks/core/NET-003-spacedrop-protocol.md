---
id: NET-003
title: Spacedrop Protocol
status: To Do
assignee: jamiepine
parent: NET-000
priority: High
tags: [networking, spacedrop, sharing, p2p, upstream-backlog]
whitepaper: Section 4.5.3
milestone: M4
last_updated: 2026-10-16
---

## Description

Implement the Spacedrop protocol for ephemeral, secure file sharing between non-paired devices. This feature will be similar to Apple's AirDrop, allowing for quick and easy file transfers between nearby devices.

## Implementation Steps

1.  Design the Spacedrop protocol, including device discovery, connection establishment, and file transfer.
2.  Implement the protocol as a new handler in the `NetworkingService`.
3.  The protocol should use a secure method for authenticating the transfer (e.g., a short code or a user confirmation).
4.  Integrate the Spacedrop functionality with the UI/CLI.

## Acceptance Criteria

- [ ] Two non-paired devices can discover each other on a local network.
- [ ] A user can initiate a file transfer to another device using Spacedrop.
- [ ] The file transfer is secure and efficient.

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It belongs to multi-device integration (M4 in
`.tasks/ROADMAP.md`), which starts only after M2 and M3. Re-check the design against current
WingDrive code before pulling it into a sprint.
