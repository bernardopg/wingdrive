---
id: DESK-035
title: Git status emblems on files
status: In Progress
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

1. A library query `files.vcs_status` takes a directory, walks up to the `.git` root and runs
   `git status --porcelain=v2 -z` scoped to it, with a timeout. It returns each direct child's
   state (untracked, added, modified, conflicted); a child folder takes the strongest state
   beneath it.
2. Outside a working tree, or when `git` is not installed, it returns nothing and costs one stat
   per parent folder.
3. Grid and list views draw a small emblem from the result; the query re-runs when the listing
   changes and when the window regains focus, so the emblems follow edits and commits.

## Acceptance Criteria

- [x] Files in a Git working tree show their status as an emblem
- [ ] Verified on the Linux desktop build

## Evidence

- `files.vcs_status` (`core/src/ops/files/query/vcs_status.rs`) stops at the first `.git` found in
  the folder or its parents, runs `git status` with `GIT_OPTIONAL_LOCKS=0` and a 5 s timeout, and
  folds the porcelain v2 records onto the folder's children. Unit tests cover every record kind,
  folder aggregation, an untracked ancestor, a folder outside a tree, and a real repository built
  with `git init`; they run in CI.
- `useExplorerFiles` returns `vcsStatus` for directory listings; `VcsEmblem` draws `U`, `A`, `M` or
  `C` on grid cards and list rows. `vcsStatusOf` is covered by `vcsStatus.test.ts`.
- Open: runtime check on the Linux desktop build inside a checkout (edit, stage, commit in a
  terminal and come back to the window).
