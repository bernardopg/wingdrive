---
id: SHARE-010
title: "Interface: Share Management View"
status: To Do
assignee: jamiepine
parent: SHARE-000
priority: Medium
tags: [interface, sharing, ui, upstream-backlog]
milestone: M4
last_updated: 2026-10-16
---

## Description

Per-Space and global "My Shares" view to inspect, edit, and revoke shares.

## Implementation Steps

1. "Active shares" tab on Space settings
2. Global "Shares" route in settings, listing across all spaces
3. Per-row actions: copy link, edit password/expiry, regenerate token (revoke old, mint new), revoke
4. Show: created date, expiry, last accessed, access count (when wingdrive.app provides them)
5. Sync indicator per row: published / pending / failed (visualizes SHARE-008 state)

## Acceptance Criteria

- [ ] User can see all active shares for a given Space
- [ ] User can revoke a share and the link stops resolving within seconds
- [ ] User can regenerate a token without losing the underlying share

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It belongs to multi-device integration (M4 in
`.tasks/ROADMAP.md`), which starts only after M2 and M3. Re-check the design against current
WingDrive code before pulling it into a sprint.
