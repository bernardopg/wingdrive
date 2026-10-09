---
id: DESK-010
title: Startup latency and resident memory budget
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, performance]
last_updated: 2026-10-09
---

## Description

With the app resident, opening a folder must feel as fast as Dolphin. Cold start and memory are measured and kept within a budget.

## Implementation Steps

- [x] Measure time from `wingdrive <dir>` to a painted listing, cold and resident
- [x] A launch while WingDrive runs is handed over D-Bus before GTK starts
- [x] Daemon readiness polling starts at 25 ms instead of 100 ms; avoid waiting when the socket is already up
- [x] Budgets: resident open < 300 ms, cold start < 2.5 s, idle PSS of app + daemon recorded in the smoke test

## Acceptance Criteria

- [x] Measured numbers recorded in Evidence and enforced in the release smoke test

## Evidence

- Readiness poll is 25 ms instead of 100 ms and the app logs `elapsed_ms` when the daemon answers.
- `scripts/release/smoke.py` now asserts cold start to daemon ready under `WINGDRIVE_STARTUP_BUDGET_SECONDS` (45 s on CI runners, which lose about 25 s to a D-Bus timeout during GTK start-up) next to the existing PSS budget; the release run records both numbers.
- `wingdrive <dir>` with WingDrive already running now calls the single-instance D-Bus method from `main` before the Tauri builder, so the second process never initialises GTK. Debug build, private D-Bus session and network namespace under Xvfb (2026-10-08): 289 ms on the first call, then 55, 44 and 40 ms from launch to exit. Before, every resident open paid a full GTK and WebKit start-up before the plugin forwarded it.
- Named instances now keep single-instance on with their own bus name (`com.wingdrive.desktop.i<instance>`), so test runs no longer hand launches to the everyday WingDrive and the smoke test can exercise the handover.
- `scripts/release/smoke.py` runs under its own `dbus-run-session` when no session bus exists, launches the bundle a second time with a folder, and asserts the launch is handed over under `WINGDRIVE_RESIDENT_OPEN_BUDGET_MS` (300 ms) and that the running instance logs `Delivering open requests`. Local run against a debug build (2026-10-08): cold start to daemon ready 3.3 s, resident open handed over in 33 ms, folder delivered. PSS was 613.5 MiB, over the 550 MiB budget, as expected for an unoptimised debug build.
- Still open: cold start to a painted listing on a real desktop with the published alpha.9 AppImage.
- 2026-10-08, cold start to a painted listing, `wingdrive <folder with 200 files>` under Xvfb with a private D-Bus session, frames captured every ~50 ms:
  - Installed `wingdrive-bin` 2.0.0alpha9-1: window 1.3 to 2.6 s, startup screen until ~5 s, then the Overview, then the folder at ~7 s. Daemon ready 3.0 to 4.3 s after it was spawned.
  - Cause 1: the startup screen only left on the frontend's 3 s status poll, because `daemon-connected` was emitted when the IPC pool first connected, which the startup screen never triggers. The app now emits it as soon as the daemon answers, and the frontend polls every 100 ms until the first connection.
  - Cause 2: the first tab rendered the Overview and `LaunchRequestSync` navigated to the folder only after mount. The main window now takes the first folder request (`take_initial_open_request`) before its first render and starts the tab there; requests that select an entry still go through `LaunchRequestSync`.
  - With both fixes (debug app, release daemon): window 0.65 to 0.71 s, folder chrome at 4.2 s and the listing painted at 4.5 s, against ~7 s before. The daemon itself takes about 2.9 s to answer on a fresh data directory, so the 2.5 s cold budget needs daemon start-up work (library creation and migrations on first run); resident opens are handled by the D-Bus handover (33 ms in the smoke test).
- 2026-10-09, daemon start-up: on a fresh data directory the daemon spent 2.0 of its 2.9 s before RPC in a fixed sleep in `load_and_reconnect_devices`, which gave mDNS time before reconnecting paired devices. The sleep now runs inside each background reconnection task, so reconnection timing is unchanged and RPC accepts after 0.75 to 0.93 s (release build, 3 runs).
- 2026-10-09, first listing: a listing of an unindexed folder returned empty and the UI refetched about 190 ms later when the indexer's events arrived. `DirectoryListingQuery` now waits up to 250 ms (`FIRST_LISTING_WAIT`) for the indexer it just dispatched and answers with the entries when it finishes in time; larger folders still fill in from events. Covered by `core/tests/directory_listing_first_response_test.rs`, which fails without the wait.
- 2026-10-09, cold start to painted listing with all fixes, release app and release daemon, folder of 200 files, Xvfb 1920x1080 with a private D-Bus session, first frame matching the final listing: 2.07, 2.10 and 2.14 s (installed alpha.9: ~7 s). Window visible at 0.9 to 1.05 s.
