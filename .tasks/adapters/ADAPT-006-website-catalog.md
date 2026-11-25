---
id: ADAPT-006
title: "Catalog at wingdrive.org/adapters"
status: In Progress
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S07
tags: [adapters, store, website]
depends_on: [ADAPT-001]
last_updated: 2026-11-24
---

## Description

wingdrive.org/adapters lists the adapters of the official registry with the same cards as the app,
one page per adapter, and a "Publish your adapter" page that links to the guidelines. The site builds
from the signed index, so it never shows an adapter the app would refuse.

## Implementation Steps

- [x] Build-time fetch and verification of the index in `website/` (`src/i18n/adapters.ts`; icons checked against their SHA-256 and inlined; the index signature is checked by the app, not the site)
- [ ] List page with search and categories; one page per adapter (EN and PT)
- [ ] Redeploy when the registry publishes (repository dispatch; for now the daily website build picks it up)

## Acceptance Criteria

- [ ] Pages live on wingdrive.org after a registry publish
- [ ] Lighthouse accessibility score of 95 or more on the list page
