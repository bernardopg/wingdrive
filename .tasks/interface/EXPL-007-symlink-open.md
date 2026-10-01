---
id: EXPL-007
title: Open Symlinks Correctly
status: To Do
assignee: bernardopg
parent: EXPL-000
priority: Medium
sprint: S02
milestone: M1
tags: [explorer, symlink]
last_updated: 2026-10-01
---

## Description

Double-clicking an entry handles directories and files but not symlinks. A link to a directory should navigate; a link to a file should open the target.

## Acceptance Criteria

- [ ] Double-click on a symlink to a directory navigates into it
- [ ] Double-click on a symlink to a file opens the target with the default app
- [ ] Broken symlink shows an explicit error toast
