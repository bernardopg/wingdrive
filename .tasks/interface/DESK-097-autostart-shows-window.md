---
id: DESK-097
title: Start at login shows the window, and starting minimized is not a setting
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S05
tags: [desktop, tray, autostart, bug]
related_tasks: [DESK-003, DESK-051]
last_updated: 2026-11-15
---

## Description

Reported by the author on 2026-11-15: after login, WingDrive opens its window on the screen instead
of starting in the tray, and how it starts is not a setting.

Found in the journal of the author's machine on 2026-11-15. At 18:45:20 the portal activated the
FileChooser backend (DESK-051), which started `wingdrive --hidden` and stayed hidden. At 18:45:30 the
XDG autostart entry ran `WingDrive --hidden`, which found the running instance and handed its launch
over ("Handed launch to the running WingDrive"). The single-instance callback calls `launch::deliver`,
which always shows the main window, so the `--hidden` of the second launch was lost.

## Implementation Steps

- [x] A forwarded launch with `--hidden` and no folders to open leaves the windows as they are.
- [x] New setting "Start minimized in the tray" (`start_minimized`, on by default) under Settings >
      General > Background, below "Start at login" and only enabled with it. Off, the login entry
      starts WingDrive with its window shown.
- [x] The Linux autostart entry and the Windows Run value carry `--hidden` only when the setting is on.
- [x] Unit tests for the forwarded launch and for both login entries.

## Acceptance Criteria

- [x] The login sequence of the author's machine, reproduced with a named instance: a first
      `--hidden` launch (as the portal starts it), then a second `--hidden` launch (as the autostart
      entry does) leaves WingDrive in the tray, with no window.
- [x] With "Start minimized in the tray" off, the login entry has no `--hidden`, so the launch
      shows the window (unit tests for the Linux entry and the Windows Run value).

## Evidence

- 2026-11-15, debug build of this branch, named instance `desk097`, Hyprland. First
  `WingDrive --hidden`: no window. Second `WingDrive --hidden`: "Handed launch to the running
  WingDrive" in the second process and "Kept WingDrive in the tray for a hidden launch" in the first;
  `hyprctl clients` still shows no WingDrive window. A third launch without `--hidden` opened the
  window, so a normal launch still brings WingDrive forward.
- `cargo test -p wingdrive --bin WingDrive -- launch:: background::`: 37 passed, including
  `a_hidden_launch_handed_over_keeps_the_window_hidden` and the login entries with and without
  `--hidden`.
- The cold login with the AUR package comes with `3.4.0`.
