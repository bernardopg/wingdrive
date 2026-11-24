---
id: ADAPT-013
title: "Thunderbird local mail adapter"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, mail, thunderbird]
related_tasks: [ADAPT-003, ADAPT-020]
last_updated: 2026-11-24
---

## Description

Index mail stored by Thunderbird without any OAuth: local folders and the IMAP cache (mbox and
maildir), with sender, recipients, subject, date, body text and attachment names. Read only; never
touch Thunderbird's files while it writes them. Real data: `~/.thunderbird` (2.1 GB).

## Implementation Steps

- [ ] Profile detection from `profiles.ini` on every platform
- [ ] mbox and maildir readers; `.msf` summaries ignored; decoding of MIME, encodings and HTML bodies to text
- [ ] Thread relations from `Message-ID`, `In-Reply-To` and `References`
- [ ] Incremental sync by mailbox offset and size; messages removed from a folder removed from the source

## Acceptance Criteria

- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input (`python3 -m unittest` under the adapter folder, run in CI)
- [ ] Rust integration test runs the real `sync.py` through `ScriptAdapter` against the fixtures and checks the stored records
- [ ] Real proof on the author's machine: sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Docs: a page in `docs/` and in the wingdrive.org user guide (EN and PT) with setup, what is indexed and privacy notes
