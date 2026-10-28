---
id: DESK-035
title: Git status emblems on files
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer]
sprint: S04
last_updated: 2026-10-28
---

## Description

Dolphin's version-control plugin marks modified, added and untracked files inside a Git checkout.
WingDrive shows no repository state.
## Implementation Steps

Next up: second easiest open M2 item. Effort: about one day, one core query plus an emblem.

1. A library query (for example `files.vcs_status`) takes a directory, walks up to the `.git`
   root and runs `git status --porcelain=v2 -z` scoped to it, with a timeout. It returns each
   entry's state (modified, added, untracked, conflicted) and an "has changes" flag for folders.
2. Outside a working tree, or when `git` is not installed, it returns nothing and costs one stat
   per parent folder.
3. Grid and list views draw a small emblem from the result; watcher events for the folder
   invalidate the query so the emblems follow edits.

## Acceptance Criteria

- [ ] Files in a Git working tree show their status as an emblem
