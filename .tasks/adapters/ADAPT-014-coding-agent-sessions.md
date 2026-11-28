---
id: ADAPT-014
title: "Coding agent sessions adapter: OpenCode, Claude Code and Codex"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, ai, agents]
depends_on: [ADAPT-004, ADAPT-008]
last_updated: 2026-11-28
---

## Description

Index the transcripts of coding agent sessions: project, title, prompts, answers, tools used and
files changed, from OpenCode (`~/.local/share/opencode`), Claude Code (`~/.claude/projects/*.jsonl`)
and Codex (`~/.codex/sessions`). Starts from the `adapters/opencode` prototype. Real data on the
author's machine: 527 MB, 651 MB and 1.4 GB.

## Implementation Steps

- [ ] Readers for the three formats, each one optional (Claude Code and Codex in 0.1.0, bernardopg/wingdrive-adapters#12; OpenCode's SQLite is left)
- [ ] One session record with message, tool call and file relations; secrets in tool output redacted (0.1.0: one record per session with tools and edited files as fields, not relations; tool output is left out and secrets in prompts and answers become `[secret]`)
- [ ] Incremental sync by file offset for append-only JSONL (0.1.0 reads a changed file again whole; second run of 1171 sessions reads 0 files)
- [ ] Large sessions summarized for search without dropping the full text (0.1.0 caps the text at 200,000 characters; 81 of 1171 real sessions hit it)

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
- [ ] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes

## Progress (2026-11-28)

`adapters/coding-sessions` 0.1.0 merged in the registry (bernardopg/wingdrive-adapters#12), built
on the adapter kit. Real data on the laptop, read only: 1171 sessions (1097 Codex, 74 Claude Code)
in 25 s, a second run reads 0 files, and a scan of the output for unredacted key shapes found none.
`wing adapter test` passes its two cases; its tests pass on Linux, macOS and Windows with Python
3.9 and 3.13.

