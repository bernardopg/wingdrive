---
id: MOB-014
title: "Fix wing library sync-setup action names and instance device lookup"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Low
milestone: M5
sprint: S05
tags: [cli, sync]
last_updated: 2026-11-09
---

## Description

`wing library sync-setup setup --help` documents `--action register-only, create-shared`, but the
parser accepts only `share-local`, `join-remote` and `merge`. With `--data-dir` and `--instance`,
the command cannot find the local device config and requires `--local-device`.

## Acceptance Criteria

- [x] Help text and accepted values match
- [x] Local device id resolved from the daemon for the selected instance

## Evidence

- 2026-11-09: `--action` is a `clap::ValueEnum` (`share-local`, `join-remote`, `merge`), so help
  and parser cannot drift; unit tests in `apps/cli/src/domains/library/args.rs`. Root cause of the
  lookup: the CLI read `device.json` from `--data-dir`, but a named instance keeps it under
  `instances/<name>`. The local device id now comes from `core.status`, and the device id and slug
  the CLI loads at startup come from the instance directory.
- Isolated daemon (`wing-daemon --data-dir <dir> --instance mob014`): before the fix
  `wing library sync-setup setup --local-library <id> --remote-device <id> --name Photos` stopped at
  `Device config not found. Please specify --local-device`; after it the request reaches the core
  (`remote_device_id: Device not found`, the id was made up). `--action register-only` is rejected
  with the list of valid values.
