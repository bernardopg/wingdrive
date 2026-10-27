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
   with the same `*`/`?` globs as Select by Pattern when the text contains them.
3. A filter bar above the view with a clear button; Escape clears and closes it.
4. Bind Ctrl+Shift+F (Ctrl+I already opens the Inspector) and add the shortcut to the shortcuts list.
5. Unit test the matcher; add the strings to `en` and `pt`.

## Acceptance Criteria

- [ ] Typing in the filter bar narrows grid, list and media views as you type
- [ ] Select all and file commands act only on the filtered items
- [ ] Escape and navigation clear the filter
- [ ] Unit tests cover the matcher
- [ ] Verified on the Linux desktop build

## Evidence

- Open.
