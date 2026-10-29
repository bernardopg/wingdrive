---
id: DESK-033
title: Act on system files as administrator
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, linux, security]
sprint: S04
last_updated: 2026-10-28
---

## Description

Dolphin (polkit admin mode) and Nautilus (`admin://`) can copy, rename or delete files the user
does not own after authenticating. WingDrive fails with a permission error.

## Implementation Steps

1. A Tauri command `run_as_admin` takes one operation (delete, rename, create folder, copy or move
   into a folder) and runs a fixed coreutils program (`rm`, `mv`, `mkdir`, `cp`) from `/usr/bin` or
   `/bin` through `pkexec`, so polkit asks for the password. No shell; every path is absolute,
   free of `.` and `..`, not the filesystem root, and passed after `--`.
2. Copies and moves never overwrite (`-n`); deletes are permanent, since root has no access to the
   user's trash.
3. When delete, rename, new folder, copy or move fail with a permission error, the toast (or the
   copy/move dialog) offers "Retry as administrator"; the listing refreshes after it succeeds.

## Acceptance Criteria

- [x] A permission error offers to retry the operation through polkit
- [ ] Verified on the Linux desktop build (needs a polkit agent and the author's password)

## Evidence

- `apps/tauri/src-tauri/src/admin.rs`: argument building, path and name checks (relative, root,
  `.` and `..`, names with `/`), the frontend's JSON shape, and a test that runs each command
  without `pkexec` on a temporary tree (create, rename, copy that skips an existing name, move,
  delete); 6 tests in the desktop crate's CI run.
- `useAdminRetry` and `isPermissionError` (`adminRetry.test.ts`); wired into rename, new folder,
  delete and the copy/move dialog.
- Open: run it with the polkit prompt on the Linux build; not done here because it needs the
  author's password.
