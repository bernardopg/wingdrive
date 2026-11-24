---
id: ADAPT-012
title: "GitHub adapter: repositories, issues, pull requests and stars"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, github, api]
depends_on: [ADAPT-004, ADAPT-008]
last_updated: 2026-11-24
---

## Description

Index the user's repositories, starred repositories, and the issues and pull requests they authored,
were assigned or review, with comments. Authenticate with a personal access token stored in the
secret store, or reuse the `gh` CLI token when present. Starts from `adapters/github`. Real data:
account `bernardopg`.

## Implementation Steps

- [ ] Token from the secret store or `gh auth token`; never written to logs or the source database
- [ ] Paginated REST or GraphQL calls with rate-limit handling and `since` cursors
- [ ] Repositories, stars, issues, pull requests and comments as related records
- [ ] Closed or transferred items updated, deleted ones removed

## Acceptance Criteria

- [ ] Lives in its own repository `bernardopg/wingdrive-adapter-<id>` created from the template (ADAPT-008), with README, license, icon and tests
- [ ] Listed in the official registry through a reviewed pull request that the registry bots (ADAPT-007) pass
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
