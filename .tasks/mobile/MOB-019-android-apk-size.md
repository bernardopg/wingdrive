---
id: MOB-019
title: "Android: cut the APK size (Rust core, ABIs, R8)"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S09
tags: [mobile, android, size, release]
related_tasks: [MOB-018, DIST-007]
last_updated: 2026-11-28
---

## Description

`WingDrive-3.8.2-android-arm64-v8a.apk` is 121 MB for one ABI and the AAB is 62 MB. IzzyOnDroid
reserves about 30 MB per app, and a large download lowers installs on Play. Most of the size is
likely the Rust core library; measure before changing anything.

## Implementation Steps

- [ ] Measure with `apkanalyzer` or `bundletool`: native libraries, Hermes bundle, resources
- [ ] Check that the mobile core is stripped and built with the size profile (`opt-level = "s"`, LTO, `panic = "abort"` if safe)
- [ ] Turn off core features the phone does not use (`wasm` is already off on mobile; check AI and media crates)
- [ ] Enable R8 (`android.enableMinifyInReleaseBuilds`) and resource shrinking, with keep rules for the JNI module
- [ ] Publish per-ABI APKs (arm64-v8a, armeabi-v7a, x86_64) instead of one universal APK

## Acceptance Criteria

- [ ] The arm64-v8a APK size is recorded before and after in this task
- [ ] The arm64-v8a APK is under 60 MB, or the task records why not
