---
id: ADAPT-014
title: "Coding agent sessions adapter: OpenCode, Claude Code and Codex"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, ai, agents]
related_tasks: [ADAPT-003]
last_updated: 2026-11-24
---

## Description

Index the transcripts of coding agent sessions: project, title, prompts, answers, tools used and
files changed, from OpenCode (`~/.local/share/opencode`), Claude Code (`~/.claude/projects/*.jsonl`)
and Codex (`~/.codex/sessions`). Extends the current `adapters/opencode` prototype. Real data on the
author's machine: 527 MB, 651 MB and 1.4 GB.

## Implementation Steps

- [ ] Readers for the three formats, each one optional
- [ ] One session record with message, tool call and file relations; secrets in tool output redacted
- [ ] Incremental sync by file offset for append-only JSONL
- [ ] Large sessions summarized for search without dropping the full text

## Acceptance Criteria

- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input (`python3 -m unittest` under the adapter folder, run in CI)
- [ ] Rust integration test runs the real `sync.py` through `ScriptAdapter` against the fixtures and checks the stored records
- [ ] Real proof on the author's machine: sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Docs: a page in `docs/` and in the wingdrive.org user guide (EN and PT) with setup, what is indexed and privacy notes
