---
id: TAURI-018
title: Explorer smoke on every pull request
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S05
tags: [tauri, explorer, testing, ci, playwright]
related_tasks: [CORE-021, TAURI-006, COV-08, COV-11]
last_updated: 2026-11-28
---

## Description

The only test that drives the Explorer end to end against a real backend is
`scripts/release/explorer.cjs`. It opens a folder with 10,002 items in Chromium through an isolated
`wing-server` and checks listing, scrolling, selection and previews. It runs only in `release.yml`,
after a tag. A regression in the interface or the core that breaks the Explorer is found at release
time, when the fix costs a new tag. That is how CORE-021 showed up: the `v3.2.0` smoke saw an empty
folder, and it passed on the second try.

The desktop app and `wing-server` serve the same `packages/interface`, so this smoke covers the
Explorer of the Tauri app too. The native shell has its own unit tests (COV-07) and the bundle
smokes in `release.yml` and `windows-bundle.yml`.

## Implementation Steps

- [x] Add a job to `ci.yml` that builds the web frontend and `wing-server`, starts the server with
      its own `--data-dir` and `--instance`, and runs `scripts/release/explorer.cjs`. Run it on pull
      requests that touch `core/`, `packages/`, `apps/web/`, `apps/tauri/` or the lockfiles.
- [x] Reuse the warm `target/` cache of the Rust jobs; keep the job under 15 minutes.
- [x] Upload the screenshots, the browser console and `wing-server` log as an artifact when it fails.
- [ ] Fail on any page error or failed RPC call that the smoke records, not only on a missing item.
- [x] Add a scheduled run that repeats the smoke 10 times on `main` and reports how many failed, so
      an intermittent failure such as CORE-021 is measured instead of retried away.
- [x] Keep `release.yml` running the same script on the release build.

## Acceptance Criteria

- [ ] A pull request that breaks the Explorer listing fails this job (checked with a deliberate
      break on a throwaway branch).
- [ ] 10 scheduled runs in a row pass on `main` after CORE-021 is fixed.
- [ ] The job is a required check on `main`.

## Progress

2026-11-15: the `Explorer smoke` job is in `ci.yml`. It runs whenever the change is not already
verified, builds the web interface and a debug `wing-server`, and repeats the smoke 10 times on
the nightly schedule, writing "N of 10 runs passed" to the run summary. Left: fail on failed RPC
calls (today only page errors fail the smoke), the deliberate-break check, 10 green nightly runs
and the required check.

2026-11-28: the scheduled runs of 2026-11-26 and 2026-11-27 each failed one of ten runs at the
missing location step. The cause was in the interface (CORE-025, second cause): a refetch of the
failed listing cleared the error for about 1.5 s. The smoke now forces that refetch with a
`visibilitychange` and fails if the alert goes away; against a build without the fix it failed with
"The missing location error disappeared during a refetch". RPC 5xx answers and page errors fail the
smoke; JSON-RPC errors in a 200 answer still do not, because the missing location and superseded
queries answer that way on purpose.
