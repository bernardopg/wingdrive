---
id: DIST-007
title: "IzzyOnDroid: decide after the size and dependency work"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Low
milestone: M5
sprint: S12
tags: [android, distribution, izzyondroid]
related_tasks: [MOB-018, MOB-019, MOB-020]
last_updated: 2026-11-28
---

## Description

IzzyOnDroid ships the developer's own APK from GitHub Releases, faster than F-Droid. Two blockers:
it keeps about 30 MB per app (the APK is 121 MB), and its inclusion policy says it is "strongly
opposed" to apps created fully or in part by generative AI tools; this repository is developed
with AI assistance (`AGENTS.md`, `.agents/skills`). Decide whether to apply at all.

## Implementation Steps

- [ ] Read the current policy at izzyondroid.org/docs/general/AppInclusionPolicy/
- [ ] If the author wants to apply and MOB-018, MOB-019, MOB-020 are done, open a request at codeberg.org/IzzyOnDroid/repodata and state the AI assistance plainly
- [ ] Otherwise close this task as not planned and record the reason

## Acceptance Criteria

- [ ] The decision is recorded here
