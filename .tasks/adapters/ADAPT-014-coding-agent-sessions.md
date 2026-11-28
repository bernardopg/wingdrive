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

- [x] Readers for the three formats, each one optional (Claude Code and Codex in 0.1.0, bernardopg/wingdrive-adapters#12; OpenCode in 0.2.0, bernardopg/wingdrive-adapters#14: `opencode.db` opened read only, `session_v2`/`session_message` since OpenCode 1.3 and the older `session`/`message`/`part` tables, child sessions left out like Claude Code sidechains, immutable open when the `-wal` file is gone)
- [ ] One session record with message, tool call and file relations; secrets in tool output redacted
      (0.2.0: `tool` and `file` records, `many_to_many` with `session`, added and removed as
      sessions change, orphans deleted; tool output is left out and secrets in prompts and answers
      become `[secret]`. Messages stay in the session's `body`, not records of their own: the app
      shows and searches only the primary model, and per-message records would store the text
      twice, 87,458 messages and 67 MB on the author's laptop. Left: message records once the app
      shows related records, ADAPT-028.)
- [x] Incremental sync by file offset for append-only JSONL (0.2.0: the state folder keeps each transcript's offset, hashes of its first 4 KB and of the 4 KB before the offset, and the parsed session, redacted and gzipped; a file that shrank or changed before the offset is read again from 0, a half-written line waits for its newline; tests cover append, truncation, rewrite and a half-written line)
- [x] Large sessions summarized for search without dropping the full text (0.2.0: no cap on `body`, which search reads, the largest real one is 2.3 M characters; a new `summary` field, the first prompt and the last answer at 400 characters each, is the preview)

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
      (built on the Python template's kit, `wingdrive_adapter.py` 1.0.0, with README EN/PT, the
      registry's Apache-2.0 license, icon and tests. Left: ADAPT-008, the template task, is To Do.)
- [x] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
      (0.2.0 through bernardopg/wingdrive-adapters#14: Structure, Code scan, Adapter tests and
      Contract harness pass, labeled `ready-for-review`; published in index 16)
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
      (2026-11-28: `wing adapters install coding-sessions --yes` on 3.8.2 installed 0.2.0 from
      index 16 on an isolated instance. Left: the desktop app install.)
- [x] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
      (Linux: the 3.8.2 daemon's detection, in its sandbox, suggested `~/.claude/projects`,
      `~/.codex/sessions` and `~/.local/share/opencode/opencode.db`. macOS: the detect test with a
      fake HOME passes on the registry's `macos-15` runners; all three defaults are the same paths
      there. A wrong path fails `sources.create` naming the field, from the manifest's `path_type`,
      `config_checks_name_the_field_a_user_filled_wrong` in `core/src/data/manager.rs`.)
- [x] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
      (Wingbot's code scan passes on 0.2.0: no host, no program, no secret; `read` lists the two
      folders and the three OpenCode database files, so `auth.json` next to them is not mounted. It
      declares no network, so the sandbox gives it none.)
- [x] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
      (2026-11-28, 3.8.2 isolated instance, 0.2.0 from the store: second sync "2 upserted, 0
      deleted in 1.2s", the two sessions being written at that moment. Updates in place and
      deletions: unit tests and both harness runs of each case, 3 of 3.)
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [x] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
      (23 unit tests on Python 3.9 and 3.13, Linux, macOS and Windows; harness cases `two-agents`,
      `opencode` and `no-agents`, 3 of 3, in bernardopg/wingdrive-adapters#14. The `opencode` case's
      database is written by `tests/opencode_db.py`, and a test checks the committed file matches.)
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
      (2026-11-28, 0.2.0 from store index 16 on an isolated 3.8.2 instance, real data read only:
      1275 sessions (82 Claude Code, 1097 Codex, 96 OpenCode), 281 tools, 2134 files and 6233 links,
      3684 lines applied in 247 s; second sync "2 upserted, 0 deleted in 1.2s"; every session has a
      date and a summary; a scan of the output for unredacted key shapes found 0. The adapter alone
      takes 45 s for the first sync and 0.2 s for the second. The instance was deleted afterwards.
      Left: opening items in the desktop app.)
- [x] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
      (2026-11-28: wingdrive.org/adapters/coding-sessions and /pt/adapters/coding-sessions list it
      with install steps and its README: Setup, Data and Privacy, and Configuração, Dados e
      Privacidade. They still show 0.1.0; the site rebuilds from the latest index, so 0.2.0 and its
      OpenCode text arrive at the next build.)

## Progress (2026-11-28, 0.2.0)

`adapters/coding-sessions` 0.2.0 merged in bernardopg/wingdrive-adapters#14 and published in index
16: OpenCode reader, resume by offset, summaries, `tool` and `file` relations. Left for Done: the
desktop app install and opening an item (screenshot), the message relation (ADAPT-028), the library
search (ADAPT-026) and the template task (ADAPT-008).

## Progress (2026-11-28, 0.1.0)

`adapters/coding-sessions` 0.1.0 merged in the registry (bernardopg/wingdrive-adapters#12), built
on the adapter kit. Real data on the laptop, read only: 1171 sessions (1097 Codex, 74 Claude Code)
in 25 s, a second run reads 0 files, and a scan of the output for unredacted key shapes found none.
`wing adapter test` passes its two cases; its tests pass on Linux, macOS and Windows with Python
3.9 and 3.13.

