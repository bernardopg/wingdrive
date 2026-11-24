---
id: ADAPT-012
title: "GitHub adapter: repositories, issues, pull requests and stars"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, github, api]
related_tasks: [ADAPT-003]
last_updated: 2026-11-24
---

## Description

Index the user's repositories, starred repositories, and the issues and pull requests they authored,
were assigned or review, with comments. Authenticate with a personal access token stored through the
secret store, or reuse the `gh` CLI token when present. Real data: account `bernardopg`.

## Implementation Steps

- [ ] Token from the secret store or `gh auth token`; never written to logs or the source database
- [ ] Paginated REST or GraphQL calls with rate-limit handling and `since` cursors
- [ ] Repositories, stars, issues, pull requests and comments as related records
- [ ] Closed or transferred items updated, deleted ones removed

## Acceptance Criteria

- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input (`python3 -m unittest` under the adapter folder, run in CI)
- [ ] Rust integration test runs the real `sync.py` through `ScriptAdapter` against the fixtures and checks the stored records
- [ ] Real proof on the author's machine: sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Docs: a page in `docs/` and in the wingdrive.org user guide (EN and PT) with setup, what is indexed and privacy notes
