---
id: DESK-005
title: Open Terminal Here
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, linux, terminal]
last_updated: 2026-10-07
---

## Description

File managers open a terminal in the current folder. WingDrive must find the user's terminal and start it there.

## Implementation Steps

- [x] Resolve terminal: `xdg-terminal-exec`, `$TERMINAL`, then known emulators (kitty, foot, alacritty, wezterm, ghostty, konsole, gnome-terminal, xterm)
- [x] Context menu on folders and empty space; shortcut
- [x] Setting to override the terminal command

## Acceptance Criteria

- [x] Open Terminal Here starts the terminal with the folder as working directory

## Evidence

- `apps/tauri/crates/file-opening-linux/src/terminal.rs` resolves an override, `xdg-terminal-exec`, `$TERMINAL`, then
  ghostty, kitty, foot, alacritty, wezterm, konsole, gnome-terminal, kgx, ptyxis, xfce4-terminal, tilix, terminator,
  xterm; 2 unit tests. `Terminal=true` desktop entries use it too.
- `open_terminal` command; macOS uses `open -a Terminal`, Windows `wt -d`.
- Settings > General has a Terminal field (empty means detect), stored in `desktop_settings.json`.
- Xvfb runtime with a probe terminal that records `pwd`: Shift+F4 in `work` and Open Terminal Here on `Sub Folder`
  recorded both directories, the second with its space intact.
- Rename now appears for a right-clicked item that is not selected and selects it first; renaming `Sub Folder` to
  `Renamed Dir` from that menu changed the folder on disk.
