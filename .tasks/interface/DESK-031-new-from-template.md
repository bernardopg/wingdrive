---
id: DESK-031
title: New file from the Templates folder
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, linux]
sprint: S04
last_updated: 2026-10-28
---

## Description

Nautilus and Dolphin list the files in `XDG_TEMPLATES_DIR` (usually `~/Templates`) under "New
Document". WingDrive creates empty files only (DESK-006).
## Implementation Steps

Next up: the easiest open M2 item. Effort: about half a day, frontend plus one small query.

1. Resolve the templates folder from `XDG_TEMPLATES_DIR` in `~/.config/user-dirs.dirs`, falling
   back to `~/Templates`; skip the submenu when the folder is missing or empty.
2. List its files with `files.directory_listing` and show them under New in the context menu and
   the toolbar, by name without extension and with the kind icon.
3. Copy the chosen template with `files.copy` into the current folder, resolve name clashes like
   the other copies do, and start the rename on the new file.

## Acceptance Criteria

- [ ] The New menu lists the templates and copies the chosen one into the current folder
