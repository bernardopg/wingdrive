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

The official adapters live in `bernardopg/wingdrive-adapters`, so the eleven prototypes under
`adapters/` leave the app repository. The ones without a finished adapter move to
`drafts/<id>/` in the registry repository, which the registry never publishes, so nothing is lost and
nothing half-built reaches users.

## Implementation Steps

- [ ] Move each prototype to `bernardopg/wingdrive-adapters` (`adapters/<id>/` when finished, `drafts/<id>/` otherwise) with a pointer to its old path
- [ ] Remove `adapters/` and the code paths that read it
- [ ] Point the docs at the registry and the template

## Acceptance Criteria

- [ ] `adapters/` gone from the main repository; every prototype reachable in its own repository
