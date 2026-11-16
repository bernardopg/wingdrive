---
id: CORE-022
title: Daemon and desktop logs flood with expected conditions
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M3
sprint: S06
tags: [core, logging, watcher, indexing, network]
related_tasks: [CORE-021, WATCH-007, NET-007]
last_updated: 2026-11-15
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

- [ ] `Permission denied` on a directory inside an ephemeral listing: count them and log one DEBUG
      line per walk with the count, not one WARN per directory. Keep WARN for the folder the user
      opened.
- [ ] A create event for a path that no longer exists is skipped at DEBUG.
- [ ] Find why a paired, trusted device fails authentication on reconnect after a successful
      connection in the same session, and fix the cause or log the reason at INFO.
- [ ] Remove the accelerator warnings: build the tray menu items without accelerators, or register
      them in the accel group GTK expects.
- [ ] Add a check to the Linux smoke that counts WARN and ERROR lines of an idle daemon over a few
      minutes with `/tmp` open, so the noise does not come back.

## Acceptance Criteria

- [ ] An idle daemon with `/tmp` open in the Explorer writes no WARN or ERROR line in 10 minutes.
- [ ] A phone that reconnects after sleep or a network change connects without "authentication failed".
