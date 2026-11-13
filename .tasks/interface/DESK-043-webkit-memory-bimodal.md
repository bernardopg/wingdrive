---
id: DESK-043
title: WebKit renderer memory is bimodal and grows while idle in the high mode
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S05
tags: [desktop, performance, memory, webkitgtk]
last_updated: 2026-11-05
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

- [x] Find what allocates while idle in the high mode (timers, animation frames, query refetch,
      image decoding) and what decides the mode at startup
- [ ] Ten packaged runs with 15 tabs stay within the 550 MiB budget and flat over 90 s idle
      (2026-11-05: flat in all ten, within budget in four; see Evidence)
- [x] `smoke.py` samples memory over time, so a slow leak fails it instead of a lucky snapshot

## Evidence

2026-11-03, packaged deb of `main` built with `tauri build --target x86_64-unknown-linux-gnu`,
run under Xvfb with the steps of `scripts/release/smoke.py`, WebKitWebProcess PSS sampled every 5 s:

- Before the fix, the high mode is a sawtooth, not a leak: 418 rising to 455, a collection drops it
  to 412, then 468 and 421, then 473 and 427, about every 70 s over 5 minutes. The app keeps making
  garbage while idle, at about 0.7 MiB/s with under 2% of one core, and the mode only says whether
  JavaScriptCore has just collected or is about to. Both builds of the comparison show it.
- The garbage came from `useDaemonStatus`: its fallback poll runs every 3 s while connected and
  called `setStatus` with a new, equal object, so `ShellWithDaemonCheck` re-rendered
  `TabManagerProvider` and all 15 tabs. `patchStatus` now returns the previous state when nothing
  changed, and React skips the render.
- After the fix, five runs held flat for 3 minutes at 271, 293, 314, 331 and 318 MiB (spread between
  launches, none rising), against 320 to 330 MiB flat or 410 to 470 MiB in the sawtooth before.
- `smoke.py` now reports each session's peak over a 30 s idle window and needs most of three fresh
  sessions under the budget.

Release runs with the new gate, bundled WebKit of the AppImage, idle peak per session:
`v2.4.0` verify 492.0 and 485.1, `v2.4.0` publish 489.6 and 496.0, `v2.4.1` 551.7, 495.9 and
478.6 MiB. All seven sessions were flat over the 30 s window (the largest drop 7 MiB, no rise),
so the idle growth is gone. What remains is spread between launches: WebKitWebProcess started at
312.8, 266.4 and 249.0 MiB in the three `v2.4.1` sessions, and one of seven sessions went over the
550 MiB budget by 1.7 MiB. With that rate the majority rule fails a release about one time in
twenty; a second high session would need a look at what the renderer holds at startup.

Open: the ten runs against the 550 MiB budget need the bundled WebKit of the AppImage, which the
release job measures. Locally the system WebKit of Arch puts the total at about 590 to 610 MiB.

2026-11-05, ten packaged runs on the author's machine (Arch, 12 cores, 38 GiB). `v3.0.0` AppImage from
the GitHub Release (checksum verified), bundled WebKit, Xvfb 1920x1080, one fresh isolated instance per
run. The script calls `session()` from `scripts/release/smoke.py` unchanged, so each run opens 15 tabs
with Ctrl+T. The only change from the release gate is the idle window, 90 s instead of 30 s, sampled
every 5 s. Total PSS of the app and its children, peak over the idle window, MiB:

| Run | Peak | WebKitWebProcess | Change over 90 s idle |
|---|---|---|---|
| 1 | 540.1 | 317.8 | -10.6 |
| 2 | 526.8 | 298.1 | -8.8 |
| 3 | 523.8 | 294.3 | -5.9 |
| 4 | 590.0 | 362.5 | -6.6 |
| 5 | 596.4 | 363.9 | -4.9 |
| 6 | 603.7 | 372.5 | -4.3 |
| 7 | 605.2 | 369.4 | -5.7 |
| 8 | 543.3 | 308.3 | -9.1 |
| 9 | 605.3 | 372.9 | -5.8 |
| 10 | 601.8 | 370.5 | -5.6 |

- Flat: no run rose by more than 0.2 MiB above its first idle sample, and every run ended lower. The
  idle growth of the high mode is gone.
- Budget: four of ten under 550 MiB. The bimodal start is still there and decides the result: the
  renderer settles at 294 to 318 MiB (total 524 to 543) or at 362 to 373 MiB (total 590 to 605),
  six times out of ten in the high mode here. WingDrive (114 to 118 MiB), the network process (about
  30) and the daemon (83 to 91) do not change between modes.
- With six in ten sessions high, the release gate (two of three sessions under 550 MiB) would fail
  more often than it passes on this machine; the `v3.0.0` release run passed on the GitHub runner.
  Next: find what the renderer allocates in the high mode at startup (the 65 MiB step), for example
  with a WebKit memory dump of one high and one low session, before touching the budget.
