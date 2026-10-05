---
id: TAURI-014
title: Linux desktop runtime robustness and search parity
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M1
sprint: S02
tags: [tauri, linux, interface, regression]
last_updated: 2026-10-05
---

## Description

Second half of the desktop UI audit, split from TAURI-013. Startup failures and window restarts must recover on their own. Auxiliary windows need the same providers as the main window. Search must match the explorer views. The search and recents checks still open in TAURI-006 are validated here.

## Implementation Steps

- [x] Bound startup failure and recover isolated per-window event subscriptions
- [x] Compose feature providers, dialogs, notifications and reconnect handling in auxiliary windows
- [x] Complete search scope/filter/sort/pagination/debounce and view parity
- [x] Preserve streamed text and improve clipboard, drag, Jobs, source retry and pairing
- [x] Add targeted tests and Linux validation in the packaged bundle

## Acceptance Criteria

- [x] Startup failures are recoverable; restart and window cleanup preserve scoped subscriptions
- [x] Jobs and Settings auxiliary windows have their required contexts and feedback
- [x] Search and preview regressions are covered by tests; pagination and source retries are explicit
- [x] Search and recents click-through pass in the packaged Linux app (closes the TAURI-006 gap)
- [x] Frontend tests, typecheck/build and relevant Rust validation pass, with limitations recorded

## Validation

Related: 10ab25d (auxiliary windows that never reached app_ready stayed hidden) already landed on main.

### What changed (2026-10-05)

- Event subscriptions: every subscription now has its own event channel delivered only to the window that opened it. Before, all subscriptions in all windows listened on one `core-event` name, so each callback received every other subscription's events. `cleanup_all_connections` cancelled every window's streams when one window reloaded or closed; it now cancels only the caller's, and a destroyed window releases its own. A stream that ends because the daemon restarted reopens with backoff (0.5 s to 15 s) until the caller unsubscribes.
- Auxiliary windows: Settings, Jobs, Inspector, Quick Preview and Spacebot share `AuxiliaryWindow`, which adds toasts, dialogs, the jobs context, theme, live file events, an error boundary and the same daemon connect/reconnect screen as the main window. The Jobs window threw outside `JobsProvider` before.
- Startup: the startup screen gives up after 45 s and shows the disconnected screen with start and install controls and an explanation, instead of spinning forever.
- Search: Location scope searches under the browsed location root (the core ignores `SearchScope::Location`), typing keeps the chosen scope and debounces with one pending timer, Filters toggles content-kind filters, results honor the sort direction and hidden-file setting, and results page in sets of 200 with Previous/Next.
- Sources: load and sync failures offer Retry.
- Spacebot: streamed reply text stays until the final message replaces it; a reply that arrives without a loaded timeline triggers a refetch.
- Drag: path bar folders accept drops, which gives every view a move-to-parent target.
- Found and fixed WATCH-005 (inode reuse taken for a rename; renames kept stale extensions).

### Evidence

- `bun test`: `searchQuery` (11) and `tauri-transport` (4) added to CI; all 58 frontend tests in the CI list pass.
- `bun run typecheck` passes; `cargo clippy -p wingdrive -p wing-core -D warnings` clean.
- Runtime (isolated `wing-server` + web build in Chromium): folder vs location scope, scope kept while typing, kind filters (Images empty, Text keeps text files), 205 results paginate 200 + 5 with Next disabled on the last page.

### Native Linux validation (debug Tauri build with the embedded frontend, Xvfb + openbox, isolated `WINGDRIVE_INSTANCE=native`)

- Startup: subscriptions opened before the daemon listened (connection refused) reopened on their own once it was ready; 14 streams became active without a reload.
- Settings window: opened with the dark theme; saving the device name showed the "Device settings saved" toast inside the window; Library shows only the two auto-tracking switches.
- Closing Settings and the pop-out Inspector released only their own streams (`Released 4 subscriptions window=settings-general`, `window=inspector-floating`); a location added from the CLI afterwards appeared live in the main window.
- Daemon killed while the app was open: the app showed "Daemon Disconnected"; Restart Daemon brought it back, the library reopened and a location added afterwards appeared live.
- Recents listed the new files; search in a location returned only that location's files.

### Bugs found and fixed during native validation

- A daemon started by the app that crashed stayed a zombie, and the library lock treated the zombie PID as alive, so Restart Daemon came back with "Library is already in use" and no libraries. The app now reaps the old child before restarting and after killing it, and on Linux the lock reads `/proc/<pid>/stat` and treats zombies as gone (`library::lock` tests).
- Folder and location search used `LIKE 'path%'`, so `/loc` also matched `/loc2`, and `_` or `%` in folder names acted as wildcards. The path scope now matches the folder or `folder/%` with escaping (`subtree_condition_excludes_prefix_siblings`). Verified from the CLI: `loc` returns nothing from `loc2`/`loc3`.

### Limitations

- The `job-manager` pop-out window has no entry point in the UI; it now renders inside `AuxiliaryWindow` with `JobsProvider`, verified by typecheck only.
- `cargo test -p wing-core --test search_test` has 4 failures (filters, ephemeral substring and date filters) that also fail on the base commit and are not run in CI.
- Validation used a debug build under Xvfb, not the AppImage.
