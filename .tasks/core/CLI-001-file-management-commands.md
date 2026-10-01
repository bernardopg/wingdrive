---
id: CLI-001
title: File Management Commands in sd-cli
status: To Do
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

- [ ] `sd-cli file rename <path> <new-name>` calls `files.rename`
- [ ] `sd-cli file delete <paths...>` moves to trash by default; `--permanent` and `--recursive` map to `FileDeleteInput`
- [ ] `sd-cli file delete --permanent` asks for confirmation unless `--yes` is passed
- [ ] `sd-cli file mkdir <parent> <name>` calls `files.createFolder`
- [ ] Each command prints a clear success line or the daemon error
- [ ] Verified against a running daemon on Linux

## Implementation Files

- `apps/cli/src/domains/file/args.rs`
- `apps/cli/src/domains/file/mod.rs`
- `core/src/ops/files/rename/input.rs`, `delete/input.rs`, `create_folder/input.rs`
