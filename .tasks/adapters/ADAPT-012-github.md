---
id: ADAPT-012
title: "GitHub adapter: repositories, issues, pull requests and stars"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, github, api]
depends_on: [ADAPT-004, ADAPT-008]
last_updated: 2026-11-28
---

## Description

Index the user's repositories, starred repositories, and the issues and pull requests they authored,
were assigned or review, with comments. Authenticate with a personal access token stored in the
secret store, or reuse the `gh` CLI token when present. Starts from `adapters/github`. Real data:
account `bernardopg`.

## Implementation Steps

- [x] Token from the secret store or `gh auth token`; never written to logs or the source database (secret config field; detect reads only the CLI user)
- [x] Paginated REST or GraphQL calls with rate-limit handling and `since` cursors
- [ ] Repositories, stars, issues, pull requests and comments as related records
- [x] Closed or transferred items updated, deleted ones removed (repositories re-listed each sync; issues updated by `since`)

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
- [ ] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
      (2026-11-28, `wingdrive-bin` 3.8.2, isolated instance, account `bernardopg` without a token:
      291 items, second run "0 upserted, 0 deleted". Update in place and removal upstream not
      checked against real data yet.)
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [x] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI (11 tests against a local fake API)
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
      (2026-11-28: `wing adapters install github` from store index 9 installed 1.0.3; the sync above.
      Left: a token run with issues and pull requests, and the desktop screenshot.)
- [ ] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
