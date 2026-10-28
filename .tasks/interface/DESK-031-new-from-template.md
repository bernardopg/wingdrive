---
id: DESK-031
title: New file from the Templates folder
status: In Progress
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

1. A library query `files.templates` resolves the folder with `dirs::template_dir()` (which reads
   `XDG_TEMPLATES_DIR` from `~/.config/user-dirs.dirs`), falling back to `~/Templates`, and lists
   the visible files in it. A folder set to the home folder disables templates, per the XDG spec.
2. The empty-space context menu shows "New from Template" with one entry per template when there
   is at least one. The Explorer has no New button in the toolbar, so the menu is the only entry.
3. A library action `files.createFromTemplate` copies the template under a free name picked like
   "New File" does, never replacing an existing file and keeping the template's permissions; the
   new file is selected and renamed in place, and Ctrl+Z removes it.

## Acceptance Criteria

- [x] The New menu lists the templates and copies the chosen one into the current folder
- [ ] Verified on the Linux desktop build

## Evidence

- `files.templates` (`core/src/ops/files/query/templates.rs`) and `files.createFromTemplate`
  (`core/src/ops/files/create_entry/mod.rs`). Unit tests cover the folder fallback, the disabled
  case, hidden files and folders skipped, a copy that never replaces, permissions kept, and a
  folder refused as a template; they run in CI.
- `useCreateEntry().newFromTemplate` and the "New from Template" submenu in
  `useEmptySpaceContextMenu`.
- Open: runtime check on the Linux desktop build with `~/Modelos` (pt) holding a document and an
  executable script.
