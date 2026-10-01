---
id: CLI-001
title: File Management Commands in sd-cli
status: Done
assignee: bernardopg
parent: CLI-000
priority: High
milestone: M1
sprint: S01
tags: [cli, file-ops]
last_updated: 2026-10-01
---

## Description

`sd-cli file` only exposes `copy` (with `--move`), `info`, and `list`. The core already registers `files.rename`, `files.delete`, and `files.createFolder`, so the CLI cannot manage files on its own. Add thin subcommands that call the existing actions.

## Acceptance Criteria

- [x] `sd-cli file rename <path> <new-name>` calls `files.rename`
- [x] `sd-cli file delete <paths...>` moves to trash by default (recursive, like the UI); `--permanent` maps to `FileDeleteInput`
- [x] `sd-cli file delete --permanent` asks for confirmation unless `--yes` is passed
- [x] `sd-cli file mkdir <parent> <name>` calls `files.createFolder`
- [x] Each command prints a clear success line or the daemon error
- [x] Verified against a running daemon on Linux

## Fixes Found During Validation

- Every CLI `SdPath::local` carried an empty device slug because client processes never register a device, so the daemon routed CLI paths as remote: delete ran "Remote deletion (0 devices)" and rename did nothing. This also affected the existing `file copy`, `file list`, `index`, `location`, and `network` commands. `SdPath::local` now falls back to the `"local"` placeholder.
- CLI file paths are made absolute before they reach the daemon, which has its own working directory.
- `files.delete` with `permanent: true` always failed with "Permanent deletion requires explicit confirmation" because the action never set `confirm_permanent`. This broke "Delete permanently" in the desktop UI too. The action now carries the confirmation its client already obtained.

## Validation (2026-10-01)

Isolated daemon (`--data-dir <scratch> --instance clitest`), relative paths from the CLI working directory:
`file mkdir . newdir` created the folder; `file rename a.txt renamed.txt` renamed; `file delete b.txt` moved to trash;
`file delete newdir --permanent` aborted on "n" and deleted with `-y`; `file copy renamed.txt --destination copy.txt` copied;
`file list .` listed the directory.

## Implementation Files

- `apps/cli/src/domains/file/args.rs`
- `apps/cli/src/domains/file/mod.rs`
- `core/src/ops/files/rename/input.rs`, `delete/input.rs`, `create_folder/input.rs`
