---
id: SEC-008
title: "CodeQL advanced setup, TruffleHog and repository rulesets"
status: Done
assignee: unassigned
priority: High
parent: SEC-000
tags: [security, ci, github]
last_updated: 2026-10-16
milestone: M1
---

## Description

On 2026-10-10 `main` had no ruleset and no branch protection: anyone with push access could push,
force-push or delete it, and release tags could be moved or deleted after publishing. CodeQL ran in
default setup, which cannot read a config file, so it scanned the vendored glib patch and generated
types, and could not use the security-extended suite. Nothing scanned commits for leaked
credentials beyond GitHub's own provider patterns.

## Implementation Steps

- [x] `.github/workflows/codeql.yml` runs CodeQL advanced setup for actions, JavaScript and
  TypeScript, Python and Rust on pull requests, pushes to `main` and weekly, with categories
  matching default setup so existing alerts carry over. Swift is left out until SEC-009.
- [x] `.github/codeql/codeql-config.yml` selects security-extended and ignores vendored, generated and
  build output.
- [x] `.github/workflows/trufflehog.yml` scans each pull request and push for verified and
  unverifiable secrets, and scans all history weekly. A local run over all 5,201 commits found none.
- [x] `.github/rulesets/main.json` protects the default branch: no deletion or force-push, changes
  through pull requests with resolved threads, the CI, TruffleHog checks and CodeQL merge protection
  for high or critical security alerts.
- [x] `.github/rulesets/release-tags.json` stops `v*` tags from being moved or deleted.
- [x] `scripts/github-settings.sh` applies the rulesets by name, the merge options and the secret
  scanning extras, and disables CodeQL default setup.
- [x] A repository admin runs `scripts/github-settings.sh` after this change merges. Applied
  2026-10-12: `gh api repos/bernardopg/wingdrive/rulesets` lists `main` and `release tags`, both
  `enforcement: active`, and code scanning default setup reports `not-configured`.

## Acceptance Criteria

- [x] A direct push or force-push to `main` is rejected.
- [x] A pull request with a failing CI job, a TruffleHog finding or a new high-severity CodeQL alert
  cannot merge.
- [x] Deleting or moving a `v*` tag is rejected.
