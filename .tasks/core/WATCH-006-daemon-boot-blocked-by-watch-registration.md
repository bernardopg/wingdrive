---
id: WATCH-006
title: Daemon at login is unreachable while location watches register
status: In Progress
assignee: bernardopg
parent: WATCH-000
priority: Critical
milestone: M2
sprint: S03
tags: [watcher, daemon, tauri, packaging, bug]
last_updated: 2026-10-19
---

## Description

After the 2.0.0 AUR install the desktop app could not reach `wing-daemon` at login. The daemon binds its RPC socket only after `Core::new` starts every service, and the watcher service waited for notify's recursive inotify walk of every location. A location on `/home/bitter/development` (358k directories, mostly `node_modules` and `target`) kept the socket closed for 154 s to 5 min 20 s.

At login the systemd user service and the XDG autostart entry both start. The app found the port closed, spawned a second daemon that exited on the library lock (`Database already open. Cannot acquire lock.`), and its setup task returned without retrying, so the window never left the disconnected screen. `start_daemon` also killed its own daemon after 30 s.

The disconnected screen lists `wing` commands, but the package shipped only `WingDrive` and `wing-daemon`.

A first fix (84b23bc4d) was pushed to `chore/release-2.0.0` after #252 merged, so 2.0.0 shipped without it. It also watched only the top directory of a tree moved into a location, so files in its subdirectories produced no events.

## Implementation Steps

- [x] Register location watches on background tasks; walk recursive roots per directory on Linux, skipping the trees the indexer's default rules exclude and never entering symlinks (from 84b23bc4d)
- [x] Walk a directory that appears under a root, not only the directory itself
- [x] Run the tree walk before taking the `watched_paths` lock
- [x] Desktop app waits for a running or starting daemon instead of giving up; skip spawning when `wingdrive-daemon.service` is active; startup timeout 30 s to 10 min
- [x] Ship the `wing` CLI as a second sidecar and link `/usr/bin/wing` in the AUR package

## Acceptance Criteria

- [x] A daemon restart with a large location binds RPC in about a second
- [x] Files created in nested directories of a tree moved into a location produce events
- [ ] The app connects at login when the systemd service is still starting (needs the next package)
- [ ] `wing` exists on the PATH after installing the package (needs the next package)

## Evidence (2026-10-19)

- Journal of the 2.0.0 install: service started 11:01:06, watch registration of `/home/bitter/development` finished 11:03:41, RPC bound 11:03:41; the app's own daemon failed at 11:01:25 on the library lock.
- `recursive_watch_covers_nested_dirs_of_a_moved_in_tree` fails on 84b23bc4d and passes on main and on this branch; `cargo test -p wing-fs-watcher --lib` 29 passed.
- Disposable instance (`--instance boottest`) with a location on `/home/bitter/development`: after a restart RPC answered in 767 ms, bound 55 ms after the watcher service started; registration finished 1.5 s later in the background with 34,339 inotify watches.
- `cargo test -p wing-core --test fs_watcher_test --test ephemeral_watcher_test --test indexing_responder_reindex_test` and `cargo test -p wingdrive` pass.
- Also seen while removing the locations from the user's library: `location remove` fails with `database is locked` while the indexer writes, and the location's indexer and thumbnail jobs keep running after removal. Not fixed here.
