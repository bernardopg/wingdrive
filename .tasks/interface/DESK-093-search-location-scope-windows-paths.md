---
id: DESK-093
title: Search in This Location searches the whole library on Windows
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S05
tags: [desktop, search, windows, bug]
last_updated: 2026-11-11
---

## Description

`findLocationRoot` (used by the search refinement bar, DESK-021) finds the location that holds the
current folder by trimming trailing `/` from each location root and testing `path.startsWith(root + "/")`.
Windows paths use `\`, so `C:\Users\me\docs` is never inside the location `C:\Users\me`. The location
scope then has no root, and `buildSearchInput` falls back to `Library`: a search scoped to the
location returns matches from every location. Found while reviewing the DESK-021 code on 2026-11-11.

## Steps to reproduce

1. On Windows, add `C:\Users\me` as a location and open `C:\Users\me\docs`.
2. Search and pick the "This Location" scope.
3. The results include files from other locations, and the scope bar says it searched the library.

## Acceptance Criteria

- [x] A location root matches folders under it with either separator, on whole segments only
- [x] `searchQuery.test.ts` covers Windows paths

## Fix

`findLocationRoot` trims either separator from the root and accepts a folder when the character after
the root is the end of the path, `/` or `\`. `C:\Users\meg` still does not match `C:\Users\me`, and a
drive root such as `D:\` holds every folder on the drive. "matches Windows paths on whole segments" in
`searchQuery.test.ts` covers it; `bun test packages/interface/src/routes/explorer/hooks/searchQuery.test.ts`
passes 14 tests. Not run on a Windows machine; the `TypeScript (Windows)` CI job runs the same test.
