---
id: DESK-010
title: Startup latency and resident memory budget
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, performance]
last_updated: 2026-10-07
---

## Description

With the app resident, opening a folder must feel as fast as Dolphin. Cold start and memory are measured and kept within a budget.

## Implementation Steps

- [ ] Measure time from `wingdrive <dir>` to a painted listing, cold and resident (resident handover measured; painted listing on a real desktop still open)
- [x] A launch while WingDrive runs is handed over D-Bus before GTK starts
- [x] Daemon readiness polling starts at 25 ms instead of 100 ms; avoid waiting when the socket is already up
- [ ] Budgets: resident open < 300 ms, cold start < 2.5 s, idle PSS of app + daemon recorded in the smoke test

## Acceptance Criteria

- [ ] Measured numbers recorded in Evidence and enforced in the release smoke test

## Evidence

- Readiness poll is 25 ms instead of 100 ms and the app logs `elapsed_ms` when the daemon answers.
- `scripts/release/smoke.py` now asserts cold start to daemon ready under `WINGDRIVE_STARTUP_BUDGET_SECONDS` (45 s on CI runners, which lose about 25 s to a D-Bus timeout during GTK start-up) next to the existing PSS budget; the release run records both numbers.
- `wingdrive <dir>` with WingDrive already running now calls the single-instance D-Bus method from `main` before the Tauri builder, so the second process never initialises GTK. Debug build, private D-Bus session and network namespace under Xvfb (2026-10-08): 289 ms on the first call, then 55, 44 and 40 ms from launch to exit. Before, every resident open paid a full GTK and WebKit start-up before the plugin forwarded it.
- Named instances now keep single-instance on with their own bus name (`com.wingdrive.desktop.i<instance>`), so test runs no longer hand launches to the everyday WingDrive and the smoke test can exercise the handover.
- `scripts/release/smoke.py` runs under its own `dbus-run-session` when no session bus exists, launches the bundle a second time with a folder, and asserts the launch is handed over under `WINGDRIVE_RESIDENT_OPEN_BUDGET_MS` (300 ms) and that the running instance logs `Delivering open requests`. Local run against a debug build (2026-10-08): cold start to daemon ready 3.3 s, resident open handed over in 33 ms, folder delivered. PSS was 613.5 MiB, over the 550 MiB budget, as expected for an unoptimised debug build.
- Still open: cold start to a painted listing on a real desktop with the published alpha.9 AppImage.
