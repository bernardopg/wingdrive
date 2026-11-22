---
id: TAURI-017
title: Administrator retries on Windows through UAC
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: Low
milestone: M3
sprint: S06
tags: [tauri, windows, permissions, file-operations]
related_tasks: [DESK-033, TAURI-016]
last_updated: 2026-11-23
---

## Description

On Linux, an operation that fails for lack of permission offers "Retry as administrator", which
runs the copy, move, rename or delete through `pkexec` and coreutils (DESK-033). Windows has no
such path: since TAURI-016 the action is hidden off Linux, because it only failed there.

Windows needs its own design. There is no coreutils, and a UAC prompt elevates a whole process.
Building an elevated `cmd` or PowerShell command line from file paths would let a crafted name run
code as administrator, so paths must never pass through a shell.

## Implementation Steps

- [x] Pick the elevated helper: a small `wing-elevate.exe` started with `ShellExecuteExW`
      and the `runas` verb, which reads its operations from a file, not from its command line
- [x] Support the same operations as DESK-033: copy, move, rename, delete, create folder
- [x] Report each operation's result back to the app, so a partial failure is shown per file
- [x] Offer the action on Windows in `apps/tauri/src/platform.ts` and the Explorer error toasts

## Acceptance Criteria

- [ ] Copying into `C:\Program Files\<folder>` without rights shows "Retry as administrator", raises
      one UAC prompt and completes the copy (implemented, not verified on Windows in this session)
- [ ] Declining the UAC prompt leaves the files untouched and shows a cancelled state, not an error
      (implemented: `ERROR_CANCELLED` maps to `AdminRunOutcome::Cancelled`, frontend shows info toast; not verified on Windows)
- [x] A file name with `&`, `|`, `"`, `%` or `^` is handled as data: test it in the helper's unit tests
- [ ] The helper is signed with the release's Authenticode certificate (REL-001)

## Notes

Created on 2026-11-05 from the open list of TAURI-016. It waits for REL-001: an unsigned
elevated helper shows an "unknown publisher" UAC prompt, which users should not learn to accept.

## Progress (2026-11-23)

### What was done

- Added `crates/wing-elevate`: protocol types (`AdminOperation`, `AdminRequest`, `AdminResponse`,
  `AdminStatus`, `AdminItemFailure`), validation, file-operation execution, and request/response
  file helpers. The `wing-elevate` binary reads `--request <file> --response <file>` and never takes
  operations from the command line.
- Updated Tauri admin command (`apps/tauri/src-tauri/src/admin.rs`) to return `AdminRunOutcome`
  (`completed | cancelled | partial`). Linux keeps the pkexec path and now maps dismissal to
  `cancelled`. Windows launches `wing-elevate` with `ShellExecuteExW`/`runas`, waits for exit, reads
  the response file, and maps UAC cancellation to `cancelled`.
- Exposed `runAsAdmin` on Windows in `apps/tauri/src/platform.ts`; updated `PlatformContext` types
  and `useAdminRetry` to handle cancelled/partial outcomes. Added i18n keys for cancelled/partial in
  `en` and `pt` common locales.
- Wired build/packaging: added wing-elevate to `build-daemon.ts`, `dev-with-daemon.ts`,
  `apps/tauri/src-tauri/build.rs` sidecar staging, and `tauri.conf.json` `externalBin`.
- Updated docs/AGENTS to reflect Linux+Windows admin retry (Windows helper unsigned until REL-001).

### Evidence

- `cargo test -p wing-elevate` (Linux): 5 passed, 0 failed. Includes special-character names
  (`& | " % ^`), request/response round trip, skip-existing transfer, partial failure reporting,
  and validation refusals.
- `cargo test -p wingdrive --bin WingDrive admin` (Linux): see PR/testing section; Linux admin tests
  updated for `AdminRunOutcome`.

### Remaining / not verified

- Windows UAC flow not manually verified in this Linux environment (no Windows machine/session).
- Authenticode signing not done; tracked by REL-001.
- Full Windows CI/build validation depends on GitHub Windows runners.
