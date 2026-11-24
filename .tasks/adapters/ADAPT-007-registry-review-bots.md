---
id: ADAPT-007
title: "Registry review checks run by Wingbot"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, ci, security, automation]
depends_on: [ADAPT-001, ADAPT-003]
related_tasks: [ADAPT-008, ADAPT-009]
last_updated: 2026-11-24
---

## Description

A pull request that adds or updates an adapter in the registry gets an automatic review before a
human looks at it. The checks run in GitHub Actions and speak as Wingbot (ADAPT-016). The bot checks structure and requirements, runs the adapter's tests and the
contract harness in a sandbox without network, scans for risky code, and comments one report on the
pull request. After a maintainer approves, the bot builds the archive, records its checksum, signs
the index and publishes it.

## Implementation Steps

- [ ] Structure: entry schema, matching `adapter.toml`, icon (SVG, square, size limit), summary length, README sections, OSI license, no binaries or vendored dependencies
- [ ] Requirements: contract version supported, tests present and passing, harness passing against the adapter's fixtures, declared platforms and runtime
- [ ] Security: declared permissions match what the code uses (network hosts, filesystem paths, secrets); no `eval`/`exec` of fetched code, no shell-outs except declared tools, no writes outside the adapter's state; dependency audit; secret scan
- [ ] The pinned commit must be reachable from a tag in the adapter repository; updates show the diff since the last pin
- [ ] Compiled adapters are built by the registry's own CI from the pinned commit for each target (Linux x86_64 and aarch64, macOS aarch64, Windows x86_64), with locked dependencies; binaries uploaded by authors are never accepted, so what users install always matches reviewed source
- [ ] One report comment, updated on every push, with a pass/fail table and links to logs; labels `needs-changes`, `ready-for-review`, `approved`
- [ ] Publish job after approval: archive, checksum, signed index, release asset, website dispatch
- [ ] Same checks runnable locally with `wing adapter validate`

## Acceptance Criteria

- [ ] A good submission and four bad ones (bad schema, failing tests, undeclared network use, unsigned update) behave as expected in the registry's CI
- [ ] Publishing an approved submission updates the signed index without manual steps
