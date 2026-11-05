---
id: TAURI-017
title: Administrator retries on Windows through UAC
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: Low
milestone: M3
tags: [tauri, windows, permissions, file-operations]
related_tasks: [DESK-033, TAURI-016]
last_updated: 2026-11-05
---

## Description

On Linux, an operation that fails for lack of permission offers "Retry as administrator", which
runs the copy, move, rename or delete through `pkexec` and coreutils (DESK-033). Windows has no
such path: since TAURI-016 the action is hidden off Linux, because it only failed there.

Windows needs its own design. There is no coreutils, and a UAC prompt elevates a whole process.
Building an elevated `cmd` or PowerShell command line from file paths would let a crafted name run
code as administrator, so paths must never pass through a shell.

## Implementation Steps

- [ ] Pick the elevated helper: a small signed `wing-elevate.exe` started with `ShellExecuteExW`
      and the `runas` verb, which reads its operations from a file or pipe, not from its command line
- [ ] Support the same operations as DESK-033: copy, move, rename, delete, create folder
- [ ] Report each operation's result back to the app, so a partial failure is shown per file
- [ ] Offer the action on Windows in `apps/tauri/src/platform.ts` and the Explorer error toasts

## Acceptance Criteria

- [ ] Copying into `C:\Program Files\<folder>` without rights shows "Retry as administrator", raises
      one UAC prompt and completes the copy
- [ ] Declining the UAC prompt leaves the files untouched and shows a cancelled state, not an error
- [ ] A file name with `&`, `|`, `"`, `%` or `^` is handled as data: test it in the helper's unit tests
- [ ] The helper is signed with the release's Authenticode certificate (REL-001)

## Notes

Created on 2026-11-05 from the open list of TAURI-016. It waits for REL-001: an unsigned
elevated helper shows an "unknown publisher" UAC prompt, which users should not learn to accept.
