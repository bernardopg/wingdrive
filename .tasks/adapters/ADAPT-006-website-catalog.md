---
id: ADAPT-006
title: "Catalog at wingdrive.org/adapters"
status: To Do
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

- [ ] Build-time fetch and verification of the index in `website/`
- [ ] List page with search and categories; one page per adapter (EN and PT)
- [ ] Redeploy when the registry publishes (repository dispatch)

## Acceptance Criteria

- [ ] Pages live on wingdrive.org after a registry publish
- [ ] Lighthouse accessibility score of 95 or more on the list page
