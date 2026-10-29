---
id: DESK-012
title: Reduce Linux WebKit compositing cost during navigation
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
tags: [desktop, performance, linux]
last_updated: 2026-10-28
sprint: S03
---

## Description

Folder navigation in the installed alpha.13 Linux desktop can keep a logical CPU busy in WebKit's compositor even for small directories. Reduce the measured graphics cost without changing indexing, the user's desktop services, or GPU security settings. This is a mitigation, not a claim that every reported stall has been fixed.

## Implementation Steps

- [x] Sample CPU, memory, disk traffic and daemon logs during real navigation
- [x] Separate JavaScript/backend activity from WebKit compositor activity by sampling individual threads
- [x] Compare Wayland and XWayland without assuming that either backend is the cause
- [x] Run repeated backdrop-filter A/B tests with the installed desktop binary
- [x] Disable backdrop filters only in the main Linux desktop window before the first React render
- [x] Add platform/window policy tests and run them in frontend CI
- [x] Validate checks and the compiled CSS policy in WebKitGTK
- [x] Build each item's file context menu on demand instead of once per visible item
- [x] Validate a rebuilt native release binary installed over the packaged app

## Acceptance Criteria

- [x] Navigation consumes less renderer CPU in repeated, same-viewport A/B runs
- [x] macOS, Windows, the web app and auxiliary windows retain their existing rendering policy
- [x] Do not disable hardware acceleration, the WebKit sandbox, transitions or animations globally
- [x] Typecheck, frontend build and regression tests pass
- [x] A release build installed in place of the package is validated against the original symptom

## Evidence

2026-10-14, installed `wingdrive-bin` 2.0.0alpha13-1, system WebKitGTK, Intel i915, Hyprland.

- A 60-second navigation capture measured DankSearch at 0.13% CPU on average (4% peak), `wing-daemon` at 1.18% (7% peak), and WebKit's renderer at 90.82% (138% peak). Here 100% means one logical CPU, not the whole machine. Renderer RSS stayed between about 725 and 737 MiB. This capture does not support DankSearch as the cause of these navigation stalls.
- A separate per-thread capture during repeated folder opens attributed about 55 to 80 percentage points of renderer CPU to `ThreadedCompositor` and 9 to 29 to the graphics-driver thread. The renderer's main thread was mostly idle between navigation changes. This points to graphics composition, not a proven React render loop.
- A fresh web frontend in the same WebKit engine, and a fresh isolated desktop session, did not consistently reproduce sustained idle CPU. Neither did simply switching between Wayland and XWayland. Do not describe an XWayland-specific root cause as confirmed.
- Background workspaces suppress rendering and invalidate idle-CPU comparisons. Early idle/visibility experiments were inconclusive and are not evidence of a fix.
- A/B navigation used the actual installed desktop executable through WebKitWebDriver. It alternated ten directory changes per run at a fixed 1896 by 995 content viewport. It warmed caches first, kept the directories and interaction sequence constant, and toggled only backdrop filters through a temporary stylesheet. All filters were restored after measuring.
- In the original user profile, normal runs took 2.93 and 2.86 seconds and consumed 4.00 and 3.55 renderer CPU-seconds. Reduced-backdrop runs took 2.88 and 2.92 seconds and consumed 2.20 and 2.23 renderer CPU-seconds. Renderer CPU fell 37% to 45%; wall-clock time did not materially improve in this paced test. Including the Tauri process, CPU fell about 21% to 31%. Do not present this as a measured end-to-end latency improvement.
- The same experiment in a copied, networking-disabled profile measured 3.76/3.99 renderer CPU-seconds normally and 2.37/1.73 without backdrop filters. No source library database was replaced.
- The compiled rendering helper and CSS were exercised in the installed WebKitGTK engine. The Linux main-window policy removed all 14 detected backdrop filters. Simulated macOS, Windows and Linux auxiliary-window policies retained all 14. This validates policy behavior, not those operating systems' native builds.
- All 81 tests in the frontend CI list passed, including eight rendering-policy regressions. Workspace typecheck, the Tauri Vite production build, task validation and `git diff --check` passed. Rust tests were not run: no Rust code changed. The frontend build still reports existing chunk-size, MapLibre BigInt-target and mixed dynamic/static-import warnings.
- Diagnostic desktop sessions, WebDriver servers, the GTK harness and the isolated daemon were stopped. The ordinary installed app was reopened without automation. The user's original daemon and DankSearch were not stopped. No package was replaced and no permanent Linux setting was changed.
- Seven ephemeral-job status update errors (`None of the records are updated`) and bounded 750 ms empty-folder polling were observed separately. Neither was established as the cause of sustained compositor CPU. They remain outside this rendering mitigation.

- Second session, same day. The WebDriver harness launched the installed `/usr/bin/wingdrive` (alpha.13) with an isolated `WINGDRIVE_INSTANCE`/`WINGDRIVE_DATA_DIR` and a synthetic 40x40 folder tree. Each run did 20 unpaced navigations (double-click into a folder, then Ctrl+ArrowLeft back), alternating variants in the same process. The 942x995 viewport was the tiled window size.
  - With backdrop filters: median 443 to 497 ms per navigation, 12.2 to 13.4 s wall and 17.0 to 18.8 renderer CPU-seconds per 20 navigations.
  - Without backdrop filters: median 64 to 73 ms, 4.4 to 4.6 s wall and 3.7 to 4.6 renderer CPU-seconds.
  - Unlike the paced test above, this measures end-to-end latency: about 6x faster navigation and about 4x less renderer CPU.
- `MALLOC_ARENA_MAX=8` (instead of the inherited `1`) and `GDK_BACKEND=wayland` (instead of XWayland) did not change either result beyond run-to-run noise. Neither is a cause.
- `tauri.linux.conf.json` already sets `transparent: false`, so window transparency is not a factor on Linux.
- The 14 filtered elements are small: TopBar circle buttons and groups (`backdrop-blur-xl`, 24px), the sidebar library switcher, and the tab bar (`backdrop-blur-sm`, 8px). The cost scales with the number of filtered layers, not their area.
- A debug build of this branch (`cargo build --bin WingDrive` with a fresh Vite `dist`) set `data-reduced-transparency` on the main window and left 0 filtered elements. It measured a median of 68 to 74 ms per navigation, matching the manual no-filter variant. This is a debug build, not the packaged release.

- After the compositor fix, per-thread sampling showed the renderer's main thread dominating (about 3.2 CPU-seconds per 20 navigations). A Chrome trace of the same build served by `wing-server` attributed about 1 s of 15 s to TanStack Query observer timers. Every grid card, list row, column item and media item mounted `useFileContextMenu`: a dozen mutations plus an "Open With" query keyed by the item's path. In the desktop app that query also called `getAppsForPaths` over IPC for every visible item on every folder change.
- `FileContextMenuProvider` now builds one menu when an item is right-clicked. It waits for the "Open With" apps before showing, so the first menu is complete. It stays mounted afterwards so menu actions keep a live owner. `SizeView` already used a single instance and is unchanged.
- Chrome, 10 navigations: 12 long tasks totalling 1271 ms before, 0 after; median navigation dropped from about 180 ms to about 60 ms.
- WebKitGTK debug build, 20 navigations: main-thread CPU fell from about 3.2 to about 1.9 seconds and renderer CPU from about 5.4 to about 3.4 seconds. The median per navigation was 50 to 56 ms (alpha.13: 443 to 497 ms).
- A right-click in the web UI listed the expected items, and Duplicate created `d01 copy` on disk; the listing refreshed. `FileContextMenuProvider.test.tsx` covers lazy construction, waiting for apps, and one show per request.
- Idle with the window visible: renderer about 1 to 2 percent of one CPU and the Tauri process 0.2 percent. No idle work was found to remove.
- The main window without backdrop filters was screenshotted in WebKitGTK; the TopBar, tab bar and sidebar keep their opaque surfaces and stay legible.

- Release build (`cargo build --locked --release -p wingdrive --bin WingDrive` over a fresh Vite `dist`) installed over `/opt/wingdrive/usr/bin/WingDrive`; only the frontend changed since alpha.13, so the packaged daemon was kept. Launched as `/usr/bin/wingdrive` with an isolated profile: 0 filtered elements, median 55 to 61 ms and p90 69 to 82 ms per navigation, 3.7 to 4.0 renderer CPU-seconds per 20 navigations (alpha.13: 443 to 497 ms median, 17 to 19 CPU-seconds).

Raw diagnostic captures were kept under `/tmp/wingdrive-perf-20261014-120526/` and `/tmp/wingdrive-native-diag/`. These paths are local evidence, not committed fixtures. The browser automation endpoints were loopback-only. Temporary profiles and styles are not a replacement for rebuilding and testing the package.
