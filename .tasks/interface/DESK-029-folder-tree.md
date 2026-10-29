---
id: DESK-029
title: Folder tree in the sidebar and expandable folders in list view
status: To Do
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

1. Expandable folder rows in list view, loading children with `files.directory_listing` on demand.
2. A collapsible folder tree panel in the sidebar rooted at the home folder and mounted volumes.
3. Both follow the hidden-files setting and keep their expansion state per tab.

## Acceptance Criteria

- [ ] A list-view folder row expands and collapses in place
- [ ] The sidebar tree follows the current folder
