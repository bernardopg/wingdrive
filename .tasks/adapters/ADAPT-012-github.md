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
- [x] Repositories, stars, issues, pull requests and comments as related records
      (1.1.0, bernardopg/wingdrive-adapters#14, index 16: each issue and pull request comment is a
      `comment` record with `belongs_to = ["item"]`, and the item's `comments` field holds their
      text for search. A changed issue has its comments listed again, so edited and deleted ones
      are noticed (ids per issue in `data_dir/comments.json`); issues left by the rate limit or the
      10 minute budget wait in the cursor. Harness case `issue-comments` replays recorded API
      answers; 4 new unit tests cover writes, edits, deletions, the switch, the rate limit and the
      budget.)
- [ ] Review comments on lines of a pull request's diff (`/repos/{repo}/pulls/{n}/comments`), which the issue comments endpoint leaves out
- [ ] Real sync of comments on account `bernardopg` with the `gh` token after 1.1.0: counts, requests used, second run
- [ ] Sources created with 1.0.x search their comments after the update (ADAPT-027)
- [x] Closed or transferred items updated, deleted ones removed (repositories re-listed each sync; issues updated by `since`)

## Acceptance Criteria

- [ ] Lives in `bernardopg/wingdrive-adapters` under `adapters/<id>/`, created from the template (ADAPT-008), with README, license, icon and tests
- [x] Published through a reviewed pull request that Wingbot's checks (ADAPT-007) pass, like any third-party adapter
      (1.0.4 through bernardopg/wingdrive-adapters#13: Structure, Code scan, Adapter tests and
      Contract harness pass, Wingbot's comment shows them; published in index 15)
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
      (2026-11-28: the user name comes from the GitHub CLI's `hosts.yml`, tested with a fake HOME on
      Linux, macOS and Windows runners. Left: `sources.create` checks paths and required fields,
      not formats, so a user name with a space only fails at the first sync ("username is not a
      GitHub user name"). A manifest `pattern` for string fields, checked by `sources.create`,
      would cover it.)
- [x] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
      (Wingbot's code scan passes: `api.github.com` is its only host, `~/.config/gh/hosts.yml` its only
      read path, `token` its only secret. It declares a host, so the sandbox gives it network; its
      harness cases replay recorded answers and pass in CI without network)
- [x] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
      (2026-11-28, `wingdrive-bin` 3.8.2, isolated instance, account `bernardopg` without a token:
      291 items, second run "0 upserted, 0 deleted". With the `gh` token, 1.0.3: first run 3994
      upserted in 195 s, 3992 items (65 owned and 239 starred repositories, 73 issues, 3615 pull
      requests: 45 open, 682 closed, 2961 merged); second run "10 upserted, 0 deleted" in 9.8 s:
      repositories pushed in that minute and the last pull request again (`since` is inclusive),
      each rewritten in place, plus one new pull request. Unstarring `earthtojake/text-to-cad`:
      "1 deleted", the record gone; starring it again brought it back. The token was in no file of
      the instance's data folder. After the store update to 1.0.4, the next sync reused the cursor:
      "10 upserted, 0 deleted".)
- [x] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
      (2026-11-28: each source has its own FTS index (`harness` matches 51 items of the token
      source). ADAPT-026: on an isolated `wing-server` with github 1.1.0 from the store, synced
      from a local fake API (synthetic user `ana`: 3 repositories, 1 star, 4 issues and pull
      requests), the list shows title, repository, date and text, and the Library search finds
      `harness` in an issue and a repository; the issue opens in its source with date and
      `issue:1101`.)
      (2026-11-28: each source has its own FTS index (`harness` matches 51 items of the token
      source), but the library search (`search.files`) does not query sources:
      `ArchiveEngine::search` has no caller in the core. ADAPT-026 adds it.)
- [x] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI (14 tests against a local fake API; since 1.0.4, five harness cases replay recorded API answers from `fixtures/<case>/api`, 5 of 5 in the registry's Validate run 46165091592)
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
      (2026-11-28: `wing adapters install github` from store index 9 installed 1.0.3; the syncs
      above, with and without a token. Left: opening items in the desktop app and its screenshot.)
- [x] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
      (2026-11-28: wingdrive.org/adapters/github and /pt/adapters/github list the adapter with install
      steps (app and `wing adapters install github`) and its README, with the Setup, Data and Privacy
      sections in English and Configuração, Dados e Privacidade in Portuguese. The site rebuilds from
      the latest index, so index 16 reaches it at the next build.)
