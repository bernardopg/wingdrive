---
id: DIST-006
title: "F-Droid: build recipe in fdroiddata and inclusion in the main repository"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S11
tags: [android, f-droid, reproducible-builds, distribution]
related_tasks: [MOB-018, MOB-020, DIST-005]
last_updated: 2026-11-28
---

## Description

F-Droid builds every app from source on its own servers, with FLOSS tools only, and rejects Play
Services (MOB-018). The build is long: Rust core with cargo-ndk, Node dependencies, Expo prebuild,
Gradle. Reproducible builds let F-Droid publish the developer-signed APK, so F-Droid, GitHub and
Play builds share one signature.

## Implementation Steps

- [ ] Wait for MOB-018 and MOB-020
- [ ] Make the Android build run from a clean checkout with one script and no secrets: pinned NDK, rustup targets, cargo-ndk, `npm ci` or a pinned Bun (ask F-Droid whether Bun is an accepted binary)
- [ ] Build it with `fdroidserver` locally (`fdroid build -v -l com.wingdrive.app`)
- [ ] Write `metadata/com.wingdrive.app.yml`: license Apache-2.0, `AntiFeatures: NonFreeNet` (Google Drive, Google Photos, OneDrive), `UpdateCheckMode: Tags`, `AutoUpdateMode: Version`, one build per ABI
- [ ] Open a merge request to gitlab.com/fdroid/fdroiddata and answer the review
- [ ] Reproducible builds: `--remap-path-prefix`, `SOURCE_DATE_EPOCH`, fixed Hermes and Gradle versions; then add `Binaries:` and `AllowedAPKSigningKeys:` to the metadata

## Acceptance Criteria

- [ ] WingDrive is in the F-Droid main repository
- [ ] The F-Droid APK has the same signature as the GitHub Release APK, or the task records why not
