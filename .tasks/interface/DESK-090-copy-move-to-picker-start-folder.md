---
id: DESK-090
title: Copy To and Move To open the folder picker outside the current folder
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S05
tags: [desktop, explorer, file-operations, bug]
last_updated: 2026-11-11
---

## Description

DESK-018 step 2 says the native folder picker opens in the current folder. `useCopyMoveTo` calls
`platform.openDirectoryPickerDialog` with a title only, and the platform API has no starting folder
for directory pickers, so the GTK portal opens in its last folder or in the home folder. You then
browse back from there to reach a folder next to the files you are copying. Found while checking
DESK-018 against its implementation steps on 2026-11-11.

## Steps to reproduce

1. In the desktop build, open a folder other than your home folder.
2. Right-click a file and choose "Copy To…".
3. The folder picker opens in another folder, not in the one the file is in.

## Acceptance Criteria

- [ ] The picker opens in the folder that holds the selection
- [x] A unit test covers the starting folder, including Windows paths

## Fix

`pickerStartFolder` in `copyMoveDestination.ts` returns the folder that holds the first selected file
(either separator, so `C:\Users\me\a.txt` gives `C:\Users\me` and `C:\a.txt` gives `C:\`).
`openDirectoryPickerDialog` takes an optional `defaultPath`, which the Tauri platform passes to the
dialog plugin, and "Copy To…", "Move To…" and "Extract To…" use it. "the picker starts in the folder
that holds the selection" in `copyMoveDestination.test.ts` covers it.
