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
last_updated: 2026-11-25
---

## Description

The English and Brazilian Portuguese catalogs list adapters from the official registry’s signed
index. Each adapter has a detail page with permissions, installation steps and documentation from
the published commit. The catalog also links to the publishing guide.

## Implementation Steps

- [x] Fetch the signed index at build time. The site checks each inline icon’s SHA-256 and the app verifies the index signature at install time.
- [x] Add search, category filters and per-adapter detail pages in English and Brazilian Portuguese.
- [x] Accept the `adapters-published` repository dispatch in the website workflow. Verify the live deployment after the registry sends the event.

## Acceptance Criteria

- [ ] Pages live on wingdrive.org after a registry publish
- [x] Lighthouse accessibility score of 95 or more on the list page. Both locales scored 100 in local preview.

## Verification

- `CI=1 bun run build` generated six adapter routes from signed index 6, covering GitHub and Obsidian in both locales. `astro check` reported zero errors and warnings.
- Chromium verified search, category filters, live counts, the catalog with JavaScript disabled and the install command copy action. Both catalog pages and both Obsidian detail pages fit viewports of 360, 390, 1280 and 1440 pixels without page-level horizontal scrolling.
- A fixture build verified Portuguese metadata, the PNG Open Graph image, the README fallback link, and the optional screenshot section. A simulated index fetch failure produced an empty catalog locally and failed the CI build.
