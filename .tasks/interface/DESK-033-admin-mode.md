---
id: DESK-033
title: Act on system files as administrator
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, linux, security]
sprint: S04
last_updated: 2026-11-05
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
- [x] Verified with the real `pkexec` and polkit on the author's machine (2026-11-05, see Evidence)

## Evidence

- `apps/tauri/src-tauri/src/admin.rs`: argument building, path and name checks (relative, root,
  `.` and `..`, names with `/`), the frontend's JSON shape, and a test that runs each command
  without `pkexec` on a temporary tree (create, rename, copy that skips an existing name, move,
  delete); 6 tests in the desktop crate's CI run.
- `useAdminRetry` and `isPermissionError` (`adminRetry.test.ts`); wired into rename, new folder,
  delete and the copy/move dialog.
- 2026-11-05, Arch Linux, polkit 128, the author's account: `scripts/check-admin-retry.sh` runs
  `admin::tests::pkexec_round_trip_on_root_owned_files`, which calls the `run_as_admin` command
  itself. A `pkttyagent` registered for the test process only answers the prompts, so the session's
  agent stays out of it. In `/opt`, which only root can write, it created a folder (owned by uid 0),
  copied a user file into it, renamed it and deleted the folder. The journal shows four
  `polkit-agent-helper` authentications and four `pkexec` commands. With a wrong password the first
  operation returns "Authentication was cancelled or refused" and nothing is created.
- The toast and dialog wiring that calls `run_as_admin` is covered by `adminRetry.test.ts`; the
  button was not clicked in the running app.
