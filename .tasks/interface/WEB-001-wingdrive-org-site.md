---
id: WEB-001
title: wingdrive.org website with automatic deploys
status: In Progress
assignee: bernardopg
priority: Medium
milestone: M3
sprint: S04
tags: [website, docs, ci, release]
last_updated: 2026-11-11
---

## Description

Public site for WingDrive at wingdrive.org, kept in the repository under `website/` so it is
maintained next to the code. Astro static site in English and Brazilian Portuguese: home with the
feature overview and animations, download page, docs, changelog, about and privacy pages.

Downloads and the changelog must never go stale: the site is rebuilt and redeployed after every
release with no manual step.

## Implementation Steps

1. Astro site in `website/`, standalone Bun project with its own lockfile (not a workspace member).
2. Art generated with Codex image generation, winged drive loop rendered in Blender, app screenshots
   from `docs/public/`.
3. `.github/workflows/website.yml`: build pull requests, deploy `main` to Hostinger over SSH, and
   redeploy after each `WingDrive Release` run and daily.
4. Hostinger website for wingdrive.org on the Cloud Startup plan, dedicated deploy key, credentials
   only in the `website` GitHub environment (limited to `main`).
5. Instructions in `AGENTS.md` ("Website").

## Acceptance Criteria

- [x] Site builds in CI and deploys from `main` without manual steps.
- [x] Download links point at the latest release assets; changelog renders the root `CHANGELOG.md`.
- [x] No secrets in the repository; gitleaks and TruffleHog clean.
- [ ] wingdrive.org resolves to the Hostinger site with a valid SSL certificate. Blocked: the .ORG
      domain order from 2026-11-11 shows as cancelled at Hostinger and the registry has no record
      of the domain, so it has to be bought again.
