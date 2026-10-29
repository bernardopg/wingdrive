---
id: DESK-037
title: Folder tree subscriptions used up the daemon's connection limit
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [interface, tauri, daemon, bug]
last_updated: 2026-10-29
---

## Description

Right after launch, the first folder opened with Ctrl+L listed "0 files"; the same folder listed correctly
on the next visit. The daemon log showed `Connection limit reached (100), rejecting new connection` in the
same second.

In the desktop app each event subscription holds its own connection to the daemon. `useNormalizedQuery`
subscribed even while its query was disabled, so every collapsed row of the sidebar folder tree (about 60
for `/` and `/tmp`) kept a connection open. The listing request then found the daemon full, and the daemon
closed it without an answer, which reached the UI as an empty listing.

## Implementation Steps

1. `useNormalizedQuery` subscribes only while its query is enabled.
2. `daemon_request` reports a connection the daemon closed without answering as an error, instead of a
   JSON parse error on an empty line.

## Acceptance Criteria

- [x] A disabled query opens no subscription
- [x] The first visit after launch lists the folder
- [ ] Verified on the packaged Linux build

## Evidence

- `normalizedQuerySubscription.test.tsx` (in CI): no subscription while disabled, one once enabled, and it
  closes when the query is disabled again. It fails without the fix.
- Runtime, 2026-10-29: debug desktop build under Xvfb with an isolated daemon. The first Ctrl+L visit to the
  fixture repository listed its four entries, the log has no `Connection limit` line, and the app held 19
  connections to the daemon (more than 100 before).
