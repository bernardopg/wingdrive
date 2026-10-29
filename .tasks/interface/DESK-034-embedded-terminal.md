---
id: DESK-034
title: Terminal panel docked in the Explorer
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, linux]
sprint: S04
last_updated: 2026-10-29
---

## Description

Dolphin's F4 opens a terminal docked under the view that follows the current folder. WingDrive only
has "Open Terminal Here" (DESK-005), which starts a separate window.

## Implementation Steps

1. Tauri runs the user's login shell on a pseudo-terminal (`portable-pty`) with the host environment
   (`file_opening_linux::host_environment_overrides`) and streams its output as raw bytes over a
   channel (`apps/tauri/src-tauri/src/terminal.rs`).
2. Following a folder types ` cd '<path>'` after Ctrl+E and Ctrl+U, only while the shell owns the
   terminal's foreground process group, so a running `vim` or build never receives the keys.
3. The interface draws the shell with xterm.js in a lazily loaded panel under the view, toggled by F4
   and by "Show Terminal Panel" in the empty-space context menu. Visibility and height are per tab.
4. Every tab renders its own router, so the terminal lives in a registry outside React and its
   element moves into each new panel: switching tabs or hiding the panel keeps the shell running.
   Closing the tab, a destroyed window, or `exit` in the shell ends it.

## Acceptance Criteria

- [x] F4 toggles a terminal panel whose working directory follows navigation
- [x] A busy terminal (a program in the foreground) does not receive `cd`
- [x] The shell survives switching tabs and hiding the panel; closing the tab ends it
- [ ] Verified on the packaged Linux build

## Evidence

- Rust tests in `terminal.rs` (bash on a real pseudo-terminal): the shell starts in the folder and
  follows to a folder whose name has a quote, a busy shell (`sleep 30`) refuses to follow, `exit 3`
  reports code 3, dropping a session hangs up its shell, and a destroyed window closes only its own
  terminals. `cd_command` quoting and control-character refusal are unit tested.
- `terminalFolder.test.ts` (only local physical folders, panel height limits) and
  `dockedTerminals.test.ts` (closing tabs hangs up their shells, focus requests) run in CI.
- Runtime, 2026-10-29: debug desktop build (`cargo build -p wingdrive` with the built frontend),
  isolated daemon (`WINGDRIVE_INSTANCE=s04`, scratch `HOME`), private D-Bus session and Xvfb `:77`,
  `GDK_BACKEND=x11`, driven with xdotool, with the user's zsh as login shell.
  - "Show Terminal Panel  F4" in the empty-space menu opened the panel with focus in the shell;
    `pwd; echo olá ação` printed the accents correctly. The shell started in `HOME`, not the
    repository: right after launch the app reconnected to its daemon and the first visit listed
    "0 files", so the view had no folder to give it. The same visit through Ctrl+L later listed the
    folder. That empty first listing also happens without the panel and is not part of this task.
  - Navigating to `/swap`, then the fixture repository, typed ` cd '/swap'` and
    ` cd '/tmp/wing-check-s04/fx/repo'`. With `sleep 15` running, opening `docs` typed nothing;
    after F4 hid and showed the panel, the history was intact and the shell caught up to `docs`.
  - A new tab (Ctrl+T) had no panel; back on the first tab the same shell and scrollback were there.
  - `exit` closed the panel; closing the tab with the panel open ended its `-zsh` (no zombie left).
- Not done: the panel does not follow `cd` typed in the shell back to the Explorer (Dolphin syncs
  both ways); a shell that was busy during navigation catches up only at the next navigation or
  remount; on Windows the terminal opens but does not follow navigation.
