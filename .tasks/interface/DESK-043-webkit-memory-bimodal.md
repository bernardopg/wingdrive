---
id: DESK-043
title: WebKit renderer memory is bimodal and grows while idle in the high mode
status: To Do
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
tags: [desktop, performance, memory, webkitgtk]
last_updated: 2026-11-03
---

## Description

The `v2.4.0` release run failed its packaged smoke test: with 15 tabs the desktop and its
children used 563.9 MiB (WebKitWebProcess 324.6 MiB) against a 550 MiB budget. The 2.2.0 and
2.3.0 runs measured 477 and 478 MiB (WebKitWebProcess about 244 MiB).

Measured locally on 2026-11-03 with the same packaged binary and the frontends of `v2.3.0`,
`14b8f1439`, `a8dc2ea31` and `v2.4.0`, the same steps as `scripts/release/smoke.py` (one
folder, 14 x Ctrl+T, 20 s):

- WebKitWebProcess lands in one of two modes on every version, about 285 MiB or about 365 MiB.
  `v2.3.0`: 289, 287, 361, 359, 281, 363. `v2.4.0`: 293, 368, 290, 368, 289, 360.
- In the low mode memory drops within 5 s of opening the tabs and stays flat.
- In the high mode it never drops and keeps growing while the app is idle: 365 to 418 MiB in
  90 s in one run, 359 to 418 MiB in another. The DOM stays at 470 nodes and the daemon logs nothing
  during that time.

So the release failure was the high mode, not a change in 2.4.0 (the failed job was rerun).
The growth in the high mode is real and would show up as the app sitting in the tray all day.

## Acceptance Criteria

- [ ] Find what allocates while idle in the high mode (timers, animation frames, query refetch,
      image decoding) and what decides the mode at startup
- [ ] Ten packaged runs with 15 tabs stay within the 550 MiB budget and flat over 90 s idle
- [ ] `smoke.py` samples memory over time, so a slow leak fails it instead of a lucky snapshot
