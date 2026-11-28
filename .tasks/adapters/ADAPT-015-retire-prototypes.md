---
id: ADAPT-015
title: "Retire the prototypes in `adapters/`"
status: Done
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S07
tags: [adapters, cleanup]
depends_on: [ADAPT-010, ADAPT-012, ADAPT-013, ADAPT-014]
last_updated: 2026-11-28
---

## Description

The official adapters live in `bernardopg/wingdrive-adapters`, so the eleven prototypes under
`adapters/` leave the app repository. The ones without a finished adapter move to
`drafts/<id>/` in the registry repository, which the registry never publishes, so nothing is lost and
nothing half-built reaches users.

## Implementation Steps

- [x] Move each prototype to `bernardopg/wingdrive-adapters` (`adapters/<id>/` when finished, `drafts/<id>/` otherwise) with a pointer to its old path (bernardopg/wingdrive-adapters#10: nine drafts copied unchanged from 93eb1dd, `drafts/README.md` points at the old path; Obsidian and GitHub were already finished in `adapters/`)
- [x] Remove `adapters/` and the code paths that read it (no code read it since ADAPT-003; the labeler, the Dependabot note and the independence check named it)
- [x] Point the docs at the registry and the template (`docs/core/design/archive-usage.md`)

## Acceptance Criteria

- [x] `adapters/` gone from the main repository; every prototype reachable in its own repository (`drafts/<id>/` in the registry)
