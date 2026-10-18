---
id: DESK-022
title: Browse archives as folders
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [core, interface]
last_updated: 2026-10-17
---

## Description

Opening a `.zip`, `.tar.*` or `.7z` should not need Xarchiver or another program. Double-clicking an
archive enters it like a folder; its files open, preview and extract from inside WingDrive.

## Implementation Steps

- [x] `engine::list` reads an archive's index (zip and 7z directories, tar headers) without extracting
- [x] `files.directory_listing` lists the folder of an archive a path points into, with implicit folders
- [x] `archive.open_member` extracts one file into `<data_dir>/cache/archives` and returns its path
- [x] `archive.extract` accepts a file or folder inside an archive and places only that
- [x] Double-click enters an archive; opening a member opens its cached copy in the default app
- [x] Quick Preview loads members from the cached copy
- [x] Inside an archive the menu offers only Open, Extract Here and Extract To; file commands are off

## Acceptance Criteria

- [x] Engine tests: listing, implicit folders, extracting one member (zip, tar.gz)
- [x] Browse tests: path splitting, cache reuse, folders refused as members
- [x] Runtime in the Tauri app: enter a zip, preview an image inside it, open a text file, extract a folder

## Evidence

- Isolated daemon: `files.directory_listing` on `test.zip` and `test.zip/docs` returned the members with
  kinds; `archive.open_member` returned the cached copy with the right content and passed a plain file through;
  extracting `test.zip/docs/img` placed `img/r.png` next to the archive.

- Tauri app on Xvfb through WebKitWebDriver (2026-10-17): entered `projeto.zip`, `.7z` and `.tar.xz`; Quick
  Preview showed `logo.png` from inside the zip; the member menu offered only Quick Look, Open, Extract Here
  and Extract To; Extract Here placed `README.md` next to the archive.

## Notes

- Members carry `File.archive` (the archive's path) from the listing, so the UI never guesses from names.
- Opened members are copied to `<data_dir>/cache/archives/<hash>/<name>` (no hidden folders, which the
  webview's asset protocol refuses); copies older than 7 days are pruned when the daemon first opens one.
  The Tauri shell allows that folder in the asset scope at runtime, wherever the instance keeps its data.
- The listing cache keeps 16 archives and is cleared when full.
