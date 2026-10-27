---
id: DESK-024
title: List view re-rendered the Explorer in an endless loop, making folder navigation take seconds
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [interface, performance, bug]
last_updated: 2026-10-27
---

## Description

In the installed 2.1.1 desktop app, every folder opened in list view took about three seconds to show its contents, while Dolphin, Nautilus and Yazi open the same folders instantly. The renderer kept one CPU core at 100% even with the window hidden in the tray.

`useTable` passed TanStack Table a new `sorting` array and a new `columnSizing` object on every render. The table therefore rebuilt its sorted row model on every render. `ListView` publishes those rows to the Explorer through `setCurrentFiles`, which stored the new array and rendered the Explorer again, which rendered `ListView` again. The loop never settled: React committed about 230 times per second.

The grid, media, column and size views publish memoized arrays and were not affected. The bug has existed since `v2.0.0-alpha.10` (6915b10cb).

## Implementation Steps

- [x] Measure the daemon's `files.directory_listing` directly over its socket
- [x] Measure navigation in the web UI and in the installed WebKitGTK app with an isolated profile
- [x] Reproduce the stall with a copy of the user's webview state against the user's daemon
- [x] Identify the render loop (DOM mutation, timer and React fiber sampling, then a CPU profile of an unminified build)
- [x] Memoize the table's `sorting` and `columnSizing` state in `useTable`
- [x] Make `setCurrentFiles` ignore a list holding the same files in the same order
- [x] Regression test in the frontend CI list
- [ ] Validate in the packaged release

## Acceptance Criteria

- [x] List view renders a stable row model when nothing changed
- [x] The hidden app with the user's state uses no CPU at idle
- [x] Folder navigation in list view takes well under a second
- [x] Typecheck and all CI frontend tests pass

## Evidence

2026-10-27, installed `wingdrive-bin` 2.1.1, system WebKitGTK, Hyprland.

- Daemon over its socket, no UI: `files.directory_listing` answered in 3 to 11 ms for cached folders, 20 to 55 ms for folders never opened, and 656 ms then 108 ms for `/usr/lib` (7,323 entries). The backend is not the bottleneck.
- Installed app with an isolated profile (grid view): 120 to 220 ms per folder. Did not reproduce.
- The user's running app: the WebKit renderer had averaged 100% of one core since login, with 1.1 GB RSS, launched with `--hidden`. Daemon events were quiet (8 events in 15 s).
- A copy of the user's webview `localStorage` (list view, hidden files shown) with the installed binary, connected to the user's daemon through a loopback forwarder: 105 to 115% CPU while hidden. A `<input>` in the PathBar changed attributes about 137 times per second; `queueMicrotask` and `postMessage` (React's scheduler) ran about 460 times per second.
- The same state in Chrome against `wing-server`, unminified build: 822 DOM mutations in 2 s visible, 390 per second in a 0x0 iframe. A CPU profile put the state updates in `ListView`'s `setCurrentFiles` effect and the `TopBarItem` registration effect.
- After the fix, Chrome: 0 mutations in 2 s, visible and in the 0x0 iframe.
- Release build of this branch (`cargo build --release -p wingdrive --bin WingDrive` over a fresh Vite `dist`; the installed `wing` copied in as the sidecar because only the frontend changed), same copied state, same daemon:
  - Hidden: 0 to 1% CPU and 0 DOM mutations, while the user's installed app measured 106% at the same moment.
  - Visible, 1902x1022, ten folder changes in `/home/bitter` (double-click, then Ctrl+ArrowLeft): installed 2.1.1 median 3107 ms and max 3241 ms; this build median 91 ms and max 240 ms.
- `bun run typecheck` and all 20 CI frontend test files pass, including the new `useTable.test.tsx`. Rust tests were not run: no Rust code changed.
- All test processes, the forwarder and the copied profile were removed. The user's app and daemon were not stopped or modified.

## Remaining navigation cost, not fixed here

- Each folder change still requests `files.directory_listing` twice, about 120 ms apart.
- Clicking a folder fetches `files.permissions` and `files.by_id` for the Inspector before the listing request starts.
- Every sidebar `SpaceItem` re-renders on each folder change (about 80 ms of 1 s of render time over seven navigations in Chrome).
- Thumbnails in `Downloads` arrive 200 to 490 ms after the listing.
