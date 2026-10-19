---
id: WATCH-007
title: Removing a busy location fails and its jobs outlive it
status: Done
assignee: bernardopg
parent: WATCH-000
priority: High
milestone: M2
sprint: S03
tags: [watcher, indexing, jobs, database, bug]
last_updated: 2026-10-19
---

## Description

Found while removing two locations from the user's library during WATCH-006.

1. `location remove` failed with `database is locked` (codes 5 and 517) while the location was indexing. The removal opened a deferred transaction that reads first, so any indexer commit in between broke it.
2. The indexer and thumbnail jobs of a removed location kept running and writing; one thumbnail job had 88k entries queued.
3. Indexing triggered by the watcher wrote entries under `target/`, which the `No Dev Directories` rule excludes. `should_filter_path` matched the event path only, and `**/target` does not match `target/debug/...`. A new directory there became a background indexer job rooted inside `target`.

Also found: a release built without the `WINGDRIVE_ANDROID_KEYSTORE_*` secrets drops the APK with only a notice, which is why 2.0.0 has no APK.

## Implementation Steps

- [x] `Job::location_scope` (indexer, thumbnail jobs dispatched for a location) kept on running jobs; `JobManager::cancel_location_jobs`
- [x] Location removal unwatches, cancels the location's jobs, then deletes in an `IMMEDIATE` transaction (`infra::db::begin_write`, shared with the indexer)
- [x] `should_filter_path` rejects a path under a rejected directory between it and the location root
- [x] Missing Android upload key is a warning in the release run

## Acceptance Criteria

- [x] Removing a location while it indexes succeeds on the first try
- [x] No job of a removed location keeps running
- [x] Watcher events under `target/` (and any rejected directory) are ignored
- [ ] The release publishes the APK (needs the upload key secrets)

## Evidence (2026-10-19)

- `location_remove_jobs_test`: the location's indexer job is cancelled, other locations' jobs are not, and removal succeeds.
- `change_detection::handler::tests`: `filters_paths_under_a_rejected_directory` fails without the ancestor check and passes with it.
- Disposable instance indexing `/home/bitter/development` in content mode: `location remove` succeeded on the first try in 2.4 s, logged `Cancelled 1 jobs`, and the daemon went quiet.
