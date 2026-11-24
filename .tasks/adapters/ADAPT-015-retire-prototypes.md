---
id: ADAPT-015
title: "Retire the prototypes in `adapters/`"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S07
tags: [adapters, cleanup]
depends_on: [ADAPT-010, ADAPT-012, ADAPT-013, ADAPT-014]
last_updated: 2026-11-24
---

## Description

Once the official adapters live in their own repositories, the eleven prototypes under `adapters/`
leave the main repository. The ones without a finished adapter yet move to repositories marked as
drafts, so nothing is lost and nothing half-built reaches users.

## Implementation Steps

- [ ] Move each prototype to `bernardopg/wingdrive-adapter-<id>` with history (`git filter-repo`) or as a fresh import with a pointer to the old path
- [ ] Remove `adapters/` and the code paths that read it
- [ ] Point the docs at the registry and the template

## Acceptance Criteria

- [ ] `adapters/` gone from the main repository; every prototype reachable in its own repository
