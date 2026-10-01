---
id: SHARE-007
title: Share Expiry & Revocation Enforcement
status: To Do
assignee: jamiepine
parent: SHARE-000
priority: Medium
tags: [sharing, lifecycle]
---

## Description

Enforce share lifecycle end-to-end: expiry, manual revocation, and propagation to wingdrive.app so dead links resolve cleanly.

## Implementation Steps

1. Expiry check on every guest handshake (cheap; same row lookup as auth)
2. Background job at a fixed interval marks expired shares and notifies wingdrive.app to remove them from the registry
3. Revoke action immediately pushes deletion to wingdrive.app and closes any in-flight guest sessions for that share
4. Distinct `expired` and `revoked` states surfaced in the management UI (SHARE-010)
5. wingdrive.app side returns a clean "no longer available" page when registry returns 404/410

## Acceptance Criteria

- [ ] Shares past `expires_at` reject guest handshakes
- [ ] Manually revoked shares close in-flight sessions within seconds
- [ ] wingdrive.app registry is updated within seconds of revoke or expiry (when core is online)
- [ ] Offline core: shares are unreachable, link resolution fails cleanly
