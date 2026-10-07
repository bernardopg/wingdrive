---
id: DESK-004
title: Real Open With on Linux
status: To Do
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

- [ ] Detect MIME type with shared-mime-info (`xdg-mime query filetype` or the `mime` database)
- [ ] Parse desktop entries from `XDG_DATA_DIRS` and `XDG_DATA_HOME`, honouring `NoDisplay`, `Hidden`, `TryExec`
- [ ] Order by `mimeapps.list` defaults and added associations
- [ ] Launch with field codes (`%f %F %u %U`) expanded, without a shell
- [ ] "Set as default" through `xdg-mime default`

## Acceptance Criteria

- [ ] Open With on a PDF lists the installed PDF viewers with the default first
- [ ] Choosing an app opens the file in it; multi-selection uses `%F`/`%U` when supported
