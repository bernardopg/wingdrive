---
id: MOB-014
title: "Fix wing library sync-setup action names and instance device lookup"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: Low
milestone: M5
tags: [cli, sync]
last_updated: 2026-10-17
---

## Description

`wing library sync-setup setup --help` documents `--action register-only, create-shared`, but the
parser accepts only `share-local`, `join-remote` and `merge`. With `--data-dir` and `--instance`,
the command cannot find the local device config and requires `--local-device`.

## Acceptance Criteria

- [ ] Help text and accepted values match
- [ ] Local device id resolved from the daemon for the selected instance
