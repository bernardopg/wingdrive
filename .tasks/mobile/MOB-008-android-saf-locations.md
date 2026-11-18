---
id: MOB-008
title: "Add locations through the Android Storage Access Framework"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S13
tags: [mobile, android, core, locations]
last_updated: 2026-11-17
---

## Description

The app cannot add a location. On Android 11 and later the core needs a folder granted through the
Storage Access Framework or `MANAGE_EXTERNAL_STORAGE`. Decide the approach and wire `locations.add`.

## Acceptance Criteria

- [x] Design note on SAF versus all-files access, including Play Store policy
- [x] A user can pick a folder, index it and browse it

## Decision (2026-11-17)

All files access (`MANAGE_EXTERNAL_STORAGE`, MOB-017), not the Storage Access Framework. The core
reads and watches real paths with `std::fs` and inotify; a SAF tree grant gives `content://` URIs
that neither works with, so SAF would need a second filesystem backend in the core. Google Play
allows the permission for apps whose core purpose is file management, which WingDrive is; the
store listing has to declare it in the permissions declaration form. Without the grant, Explore
shows the "Allow access to files" card.

## Evidence (2026-11-17)

On the moto g(7): Explore, Internal storage, `WingDriveTest`, header menu, Add as Location. The
core indexed it in 0.64 s (20 files, 8 folders), folder sizes appeared, and changes made later in
its subfolders reach the index. That last part needed a watcher fix: a folder browsed before it
became a location kept its shallow watch, so subfolder changes were missed until a restart.
