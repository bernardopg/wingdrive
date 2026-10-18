---
id: NET-006
title: Paired devices with the same hostname
status: Done
assignee: bernardopg
parent: NET-000
priority: High
tags: [networking, addressing]
milestone: M2
sprint: S03
last_updated: 2026-10-18
---

## Description

Device slugs come from hostnames. A paired device announcing this device's slug, or another peer's,
was unreachable: path resolution always picked this device, and the overview showed it under this
device's name with this device's locations.

## Implementation Steps

- [x] Colliding peers are cached under `<slug>-<first 6 hex of their UUID>`, stable across restarts
- [x] Device listings show the slug this device reaches the peer at
- [x] Paths sent to a peer carry its UUID, which every device recognises as itself

## Acceptance Criteria

- [x] Unit test for collision with this device, with another peer, and an empty slug
- [x] Two daemons named `archlinux` paired: the peer lists as `archlinux-0cd0be` and receives files
