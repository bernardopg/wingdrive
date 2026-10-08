---
id: DESK-004
title: Real Open With on Linux
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, linux, mime]
last_updated: 2026-10-07
---

## Description

`get_apps_for_file` returns an empty list on Linux, so Open With is empty. It must list the applications that handle the file's MIME type from the XDG desktop entries and `mimeapps.list`, default first, and launch the chosen one.

## Implementation Steps

- [x] Detect MIME type with shared-mime-info globs first (like GIO), then `xdg-mime query filetype` / `file`
- [x] Parse desktop entries from `XDG_DATA_DIRS` and `XDG_DATA_HOME`, honouring `NoDisplay`, `Hidden`, `TryExec`
- [x] Order by `mimeapps.list` defaults and added associations
- [x] Launch with field codes (`%f %F %u %U`) expanded, without a shell
- [x] "Set as default" through `xdg-mime default`

## Acceptance Criteria

- [x] Open With on a PDF lists the installed PDF viewers with the default first
- [x] Choosing an app opens the file in it; multi-selection uses `%F`/`%U` when supported

## Evidence

- `apps/tauri/crates/file-opening-linux/src/{desktop_entry,mime_apps}.rs`: desktop entries from `XDG_DATA_HOME` and
  `XDG_DATA_DIRS` (user copies override, `Hidden`/`TryExec` respected), `mimeapps.list` with desktop-specific files,
  subclass and alias lineage, Exec quoting and `%f %F %u %U %i %c %k`, `Terminal=true` through the terminal resolver.
  11 unit tests pass.
- On this machine `/etc/hosts` lists the DMS notepad, Neovim, LibreOffice Writer, Okular, Micro and Vim in 72 ms.
- `README.md` resolves to `text/markdown` from its glob; content sniffing alone said `text/html`.
- Xvfb runtime with an isolated `XDG_DATA_HOME` probe app: Open With listed Probe Viewer among the system editors;
  choosing it ran `probe.sh` with `my notes.txt` (space intact) as one argument. Always Open With > Probe Viewer wrote
  `text/plain=probe.desktop` to `mimeapps.list`, and Open With then showed `Probe Viewer (default)` first. With no
  default configured no entry is marked default.
- Reveal labels read Show in Finder / Show in Explorer / Show in Folder per platform.
