---
id: DESK-029
title: Folder tree in the sidebar and expandable folders in list view
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer]
sprint: S04
last_updated: 2026-10-28
---

## Description

Windows Explorer, Thunar and Dolphin's Folders panel show a folder tree; Finder and Dolphin let a
list row expand in place to show a folder's children. WingDrive has neither: the sidebar lists
locations and volumes, and list rows cannot expand.

## Implementation Steps

1. Expandable folder rows in list view: a chevron (or Right and Left) opens a folder in place;
   its children load with `files.directory_listing` through `useLibraryQueries`, sort with the
   list's columns and nest as TanStack sub-rows. Collapsing a folder also collapses those inside.
2. A "Folders" section at the bottom of the sidebar, rooted at the home folder and the filesystem
   root (the root alone where the platform cannot tell the home folder, as in the web UI). It
   opens the branch of the folder the active tab shows and highlights it; click navigates and
   middle-click opens a new tab.
3. Both follow the hidden-files setting. List expansion is stored per tab (`listExpanded`), so it
   survives reloads and restores when you come back to the folder.

## Acceptance Criteria

- [x] A list-view folder row expands and collapses in place
- [x] The sidebar tree follows the current folder
- [ ] Verified on the Linux desktop build

## Evidence

- Unit tests: `folderTreePaths.test.ts` (containment, ancestors, deepest root) and
  `listExpansion.test.ts` (toggle, attach level by level, blocks for grouping); both in CI.
- Runtime, 2026-10-28: web UI on an isolated `wing-server` (Chrome DevTools). The tree opened
  `/tmp/.../rt/fx/repo` level by level and marked it; in list view `src` and `src/deep` expanded
  with indentation, Right and Left opened and closed `docs`, the expansion survived a reload, and
  "Show in Groups" kept each expanded folder's rows under its section. No console errors.
- Found and fixed during that check: the table's core row model only rebuilds when `data`
  changes identity, so new children did not appear until the array was renewed.
- Not done: expanded children show no Git emblem (the status query covers the shown folder
  only), and the status bar counts the expanded rows too.
