---
id: CORE-022
title: Daemon and desktop logs flood with expected conditions
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M3
sprint: S06
tags: [core, logging, watcher, indexing, network]
related_tasks: [CORE-021, WATCH-007, NET-007]
last_updated: 2026-11-22
---

## Description

Review of the author's journal on 2026-11-15 (`wingdrive-daemon.service` and the desktop app, one
boot of about 4.5 hours, `v3.3.1` from the AUR). No crash, no panic, and nothing in the app's log
above INFO. The daemon wrote 1,569 lines, and most WARN and ERROR lines report conditions that are
expected on a normal desktop:

- 536 `WARN ... ephemeral::writer: Failed to read directory /tmp/systemd-private-*-<service> for
  ephemeral indexing: Permission denied (os error 13)`. `/tmp` was open in the Explorer, and a
  systemd timer recreates a private directory owned by root about every minute. Each one logs a
  warning. Folders the user cannot read are normal in `/tmp`, `/var` and other users' homes.
- `ERROR ... change_detection::handler: Failed to handle create for /tmp/sh-thd.*: No such file or
  directory`: a file that is created and deleted before the handler reads it. This is the normal
  life of a temporary file, not an error.
- `[NETWORKING ERROR] Failed to establish connection: ... authentication failed` five times in a
  row while the phone (moto g(7), `v3.3.1`) was reconnecting, after it had connected successfully
  earlier in the same boot. Either the reconnect uses stale credentials or the message hides a
  normal rejection; it needs an investigation, not only a quieter log.
- An isolated release `wing-server` running the Explorer smoke 15 times (CORE-021) logged 45
  `WARN notify::inotify: Unable to remove the path from the watches: WatchNotFound` for fixture
  folders already deleted, `WARN wing_task_system::worker::runner: Task done channel closed` from
  every worker after each walk, and `ERROR wing_fs_watcher::watcher: Failed to send raw event:
  channel closed` six times. All three describe normal teardown.
- The desktop app logs ten `Gtk-CRITICAL`-style lines `no accelerator (…) installed in accel group`
  when it builds the tray menu, on every start.

Noise hides the next real failure, and the journal is the first thing a user sends in a bug report.

## Implementation Steps

- [x] `Permission denied` or `NotFound` on a directory reached by the ephemeral writer logs at
      DEBUG; other read errors keep the WARN.
- [x] A create or modify event for a path that no longer exists is skipped at DEBUG; a real
      failure on a path that still exists keeps the ERROR.
- [x] "No entries found for device" is DEBUG: a device without locations owns no entries.
- [x] The file watcher drops raw events at DEBUG once its receiver is closed during shutdown; a
      full buffer is still an ERROR.
- [x] Find why a paired, trusted device fails authentication on reconnect after a successful
      connection in the same session, and fix the cause or log the reason at INFO.
- [x] On Linux and Windows the hidden native menu is no longer built with accelerators. Building
      it with accelerators and never attaching it to a displayed window made GTK log "no
      accelerator installed in accel group" on startup; the menu items kept in state for
      enable/disable are now created without accelerators and an empty menu is installed.
- [x] `scripts/release/daemon_smoke.py` waits for an idle window (30s default,
      `WINGDRIVE_DAEMON_SMOKE_IDLE_SECONDS`) and fails if own targets emit WARN/ERROR. The
      Explorer smoke CI step also fails if `/tmp/wingdrive-server.log` contains WARN/ERROR from own
      targets. This does not reproduce the full "/tmp open in the Explorer for 10 minutes" case.

The `notify::inotify` and `wing_task_system` lines above only appear with a `RUST_LOG` that
enables those crates. The daemon's default filter (`wing_core=info,wingdrive=info`) hides them, so
the author's journal never had them.

## Acceptance Criteria

- [ ] An idle daemon with `/tmp` open in the Explorer writes no WARN or ERROR line in 10 minutes.
      Smoke guards were added (daemon idle window + Explorer smoke server-log scan), but no 10-minute
      `/tmp`-open reproduction was run in this change.
- [x] A phone that reconnects after sleep or a network change connects without "authentication failed".

## "authentication failed" (2026-11-16)

Not the device's credentials. The message comes from the QUIC stack (`noq-proto`,
`Endpoint::handle_first_packet`): an *Initial* packet whose AEAD tag does not verify is rejected
with `PROTOCOL_VIOLATION("authentication failed")` before any TLS or device identity check. In the
author's journal the phone's connection was up the whole time (connected 18:49, lost 22:36), and the
five failures at 19:28 and 19:34 were extra handshakes arriving next to it: duplicated, late or
cross-path Initial packets from the phone's path probing. The event loop logged every failed
incoming handshake as an ERROR; it now logs them at DEBUG ("Incoming connection rejected during the
handshake"), since the dialing side reports its own failure and nothing is lost.

## Follow-up (2026-11-22, fix/core-022-daemon-log-noise)

- `apps/tauri/src-tauri/src/main.rs`: on Linux/Windows `setup_menu` no longer builds a native menu
  with accelerators. It creates only the MenuState items (duplicate/rename/delete/cut/copy) without
  accelerators and installs an empty menu. This targets the GTK "no accelerator installed in accel
  group" startup warnings; not verified on a live GTK display in this environment.
- `scripts/release/daemon_smoke.py`: added idle window (default 30s, env
  `WINGDRIVE_DAEMON_SMOKE_IDLE_SECONDS`) and WARN/ERROR scan for own targets. The check fails on
  noise before printing the captured log.
- `.github/workflows/ci.yml`: Explorer smoke now fails when `/tmp/wingdrive-server.log` contains
  WARN/ERROR from own targets.
- Evidence: `cargo check -p wing-tauri-core` and `cargo check -p wingdrive` pass on Linux. Full 10
  minute "/tmp open in Explorer" acceptance was not executed.
