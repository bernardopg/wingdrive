---
id: EXPL-007
title: Open Symlinks Correctly
status: Done
assignee: bernardopg
parent: EXPL-000
priority: Medium
sprint: S01
milestone: M1
tags: [explorer, symlink]
last_updated: 2026-10-02
---

## Description

Double-clicking an entry handles directories and files but not symlinks. A link to a directory should navigate; a link to a file should open the target.

## Acceptance Criteria

- [x] Double-click on a symlink to a directory navigates into it
- [x] Double-click on a symlink to a file opens the target with the default app
- [x] Broken symlink shows an explicit error toast

## Verification (2026-10-02)

Native tests cover links to files/directories and broken links. Grid, List, Column, Media, context menus and keyboard opening share useOpenFile.
