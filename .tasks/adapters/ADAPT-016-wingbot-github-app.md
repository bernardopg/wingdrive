---
id: ADAPT-016
title: "Wingbot GitHub App"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, automation, github, bot]
related_tasks: [ADAPT-007, ADAPT-017]
last_updated: 2026-11-24
---

## Description

Wingbot already posts on Discord. On GitHub it becomes a GitHub App owned by the author's account,
installed on `bernardopg/wingdrive` and `bernardopg/wingdrive-adapters`. Workflows mint a short-lived
installation token with `actions/create-github-app-token`, so review comments, labels, publish
commits and release notes come from `wingbot[bot]` with its own avatar instead of
`github-actions[bot]`. The logic stays in GitHub Actions, which costs nothing on public repositories;
there is no server to run.

## Implementation Steps

- [ ] Create the GitHub App "Wingbot" (name, avatar, description, homepage wingdrive.org) with the smallest permissions: contents write (registry publish), pull requests and issues write, checks write, metadata read; no organization or account permissions
- [ ] Store the App ID as a variable and the private key as a secret in a protected environment of each repository; record the key backup in the author's password manager
- [ ] Shared action `wingbot/act` in the registry repository that mints the token and posts or updates one comment per pull request
- [ ] Slash commands in pull request comments from maintainers only: `/wingbot recheck`, `/wingbot publish`, `/wingbot reject <reason>`
- [ ] Same identity used by the app repository for release notes and dependency summaries where it helps

## Acceptance Criteria

- [ ] A registry pull request shows Wingbot's report comment and labels set by `wingbot[bot]`
- [ ] A non-maintainer's `/wingbot publish` is ignored and answered with an explanation
- [ ] Rotating the App private key is documented and tested once
