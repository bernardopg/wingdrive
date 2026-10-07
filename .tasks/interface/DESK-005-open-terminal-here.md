---
id: DESK-005
title: Open Terminal Here
status: To Do
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

- [ ] Resolve terminal: `xdg-terminal-exec`, `$TERMINAL`, then known emulators (kitty, foot, alacritty, wezterm, ghostty, konsole, gnome-terminal, xterm)
- [ ] Context menu on folders and empty space; shortcut
- [ ] Setting to override the terminal command

## Acceptance Criteria

- [ ] Open Terminal Here starts the terminal with the folder as working directory
