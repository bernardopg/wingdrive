---
id: DESK-026
title: Filter the current folder by name
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard]
last_updated: 2026-10-27
---

## Description

Dolphin (Ctrl+I, filter bar), Nautilus (type to filter) and Yazi (`f`) narrow the visible items of
the folder you are in without running a search. WingDrive has typeahead, which only jumps to a
name, and a search that queries the index. A filter must act on the loaded listing at once, with
no daemon round trip, and every command (select all, copy, Quick Preview) must see only the
filtered items.

## Implementation Steps

1. Keep the filter text in the Explorer context, scoped to the folder it was typed in, so
   navigating away clears it.
2. Apply it in `useExplorerFiles` to directory and virtual listings, case-insensitive substring,
   with the same `*`/`?` globs as Select by Pattern when the text contains them. Grid and list
   views show that listing; media, size and column views list folders their own way, so the bar
   only appears in grid and list, and switching views closes it.
3. A filter bar above the view with a clear button; Escape clears and closes it.
4. Bind Ctrl+Shift+F (Ctrl+I already opens the Inspector) and add the shortcut to the shortcuts list.
5. Unit test the matcher; add the strings to `en` and `pt`.

## Acceptance Criteria

- [x] Typing in the filter bar narrows grid and list views as you type
- [x] Select all and file commands act only on the filtered items
- [x] Escape and navigation clear the filter
- [x] Unit tests cover the matcher
- [ ] Verified on the Linux desktop build

## Evidence

- `folderFilter.ts` (`filterFiles`) reuses `globMatcher` from Select by Pattern; `folderFilter.test.ts`
  covers blank text, substring and glob cases.
- The filter text lives in the Explorer context keyed by tab, pathname and search, so navigating or
  switching tabs drops it. `useExplorerFiles` applies it to directory and virtual sources and reports
  `unfilteredCount`; views publish the filtered list as `currentFiles`, which every command reads.
- `FolderFilterBar` opens with Ctrl+Shift+F (`explorer.filterFolder`, in the shortcuts list), shows
  "M of N", and closes on Escape or the X button. `bun run typecheck` and `bun test src/routes/explorer`
  (60 pass) are green.
- Open: runtime check on the Linux desktop build.
