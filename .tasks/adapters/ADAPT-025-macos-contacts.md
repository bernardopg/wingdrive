---
id: ADAPT-025
title: "macOS Contacts adapter"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Low
milestone: M5
sprint: S09
tags: [adapters]
related_tasks: [ADAPT-003]
last_updated: 2026-11-24
---

## Description

Index contacts from macOS Contacts. Needs contacts permission and a Mac to prove. A prototype exists in `adapters/`.

## Acceptance Criteria

- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input (`python3 -m unittest` under the adapter folder, run in CI)
- [ ] Rust integration test runs the real `sync.py` through `ScriptAdapter` against the fixtures and checks the stored records
- [ ] Real proof on the author's machine: sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Docs: a page in `docs/` and in the wingdrive.org user guide (EN and PT) with setup, what is indexed and privacy notes
