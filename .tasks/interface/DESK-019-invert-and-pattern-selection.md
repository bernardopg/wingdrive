---
id: DESK-019
title: Invert selection and select by pattern
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer, selection, keyboard]
last_updated: 2026-11-12
---

## Description

Explorer has Select All but no way to invert the selection or select by name pattern, which
Dolphin (Ctrl+Shift+A / Ctrl+Shift+I) and Nautilus (Ctrl+S, "Select Items Matching") both offer for
bulk work in large folders. Selection already lives in `SelectionContext` with `selectAll`.

## Implementation Steps

1. Add `invertSelection` next to `selectAll` in the selection context, over the loaded listing.
2. Bind Ctrl+Shift+I in `useExplorerKeyboard` and add the action to the Edit menu or toolbar menu.
3. Add "Select by Pattern…" (Ctrl+S): a small dialog with a glob such as `*.jpg`, case-insensitive,
   that replaces the selection with the matching names. Several globs are separated by `;`, because
   names often contain spaces and commas.
4. Unit test the glob matcher and the inversion; add the strings to `en` and `pt`.

## Acceptance Criteria

- [x] Ctrl+Shift+I inverts the selection in grid, list and column views
- [x] Ctrl+S selects every item whose name matches the glob
- [x] Unit tests cover the matcher and inversion
- [x] Verified on the Linux desktop build

## Evidence

- `selectionPattern.ts` holds the glob matcher (`*`, `?`, `[abc]`, `[!abc]`, `;`-separated globs, regex
  characters literal) and the inversion; `selectionPattern.test.ts` covers both and runs in CI.
- `useSelectionCommands` acts on the displayed collection (`currentFiles`), so all views share it.
  Ctrl+Shift+I and Ctrl+S are registered keybinds (shown in the shortcuts list), and the empty-space
  context menu gains Select All, Invert Selection and Select by Pattern…
- Packaged build, 2026-11-02: `WingDrive_2.3.0_amd64.deb` built from `main` (0c337f746) with this release's interface fixes, using the release command (`tauri build --target x86_64-unknown-linux-gnu`), extracted and run under Xvfb with an isolated HOME, data directory and instance, driven through tauri-driver and WebKitWebDriver. Ctrl+Shift+I inverted a one-item selection to the other 11 items; Ctrl+S with `*.JPG` selected exactly `photo-one.jpg` and `photo-two.jpg` (case-insensitive).

- Re-check, 2026-11-12, Isolated `wing-server` (debug build, own data dir and instance, port 18093), web UI in headless Chrome driven over CDP, fixture folder with 5 folders and 6 files: Ctrl+S with `*.JPG` selected `photo-one.jpg` and `photo-two.JPG`
  (status bar "2 selected · 6 B"), `*.md; *.txt` selected 3, `photo-[!o]*` selected only `photo-two.JPG`;
  Ctrl+Shift+I inverted to 10 items in grid view and in column view. In list view with `sub` expanded,
  Ctrl+A selected 14 rows and Ctrl+Shift+I left none selected.
- The glob matcher turned `[!]` into a class matching any character and closed `[]]` too early. Fixed in
  DESK-091 (merged with the invalid-range fallback of DESK-060).
