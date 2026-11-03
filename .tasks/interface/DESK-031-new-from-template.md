---
id: DESK-031
title: New file from the Templates folder
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, linux]
sprint: S04
last_updated: 2026-11-02
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
- [x] Verified on the Linux desktop build

## Evidence

- `files.templates` (`core/src/ops/files/query/templates.rs`) and `files.createFromTemplate`
  (`core/src/ops/files/create_entry/mod.rs`). Unit tests cover the folder fallback, the disabled
  case, hidden files and folders skipped, a copy that never replaces, permissions kept, and a
  folder refused as a template; they run in CI.
- `useCreateEntry().newFromTemplate` and the "New from Template" submenu in
  `useEmptySpaceContextMenu`.

- Runtime, 2026-11-02: isolated Linux Tauri debug build with `XDG_TEMPLATES_DIR` set to `~/Modelos`. The menu listed a document and an executable script with accented names. Copies preserved bytes and mode 751, chose a free name on collision and entered inline rename. Ctrl+Z removed the unchanged copy and preserved a copy edited after creation. The desktop undo hashes the original template before copying and checks the new file's identity and hash before removal.
- `undo::tests::undo_template_keeps_changed_or_replaced_files` covers edited and replaced files; the existing empty-file and symlink checks still pass.
- Packaged build, 2026-11-02: `WingDrive_2.3.0_amd64.deb` built from `main` (0c337f746) with this release's interface fixes, using the release command (`tauri build --target x86_64-unknown-linux-gnu`), extracted and run under Xvfb with an isolated HOME, data directory and instance, driven through tauri-driver and WebKitWebDriver. "New from Template" in the empty-space menu listed `Empty Note.txt` from `~/Templates` and created the copy.
