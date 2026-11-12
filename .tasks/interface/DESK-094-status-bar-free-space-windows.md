---
id: DESK-094
title: Status bar never shows free space on Windows
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S05
tags: [desktop, explorer, windows, bug]
last_updated: 2026-11-11
---

## Description

The Explorer status bar (DESK-027) finds the volume of the current folder with `volumeForPath`, which
tests `path.startsWith(mountPoint + "/")` after trimming trailing `/`. Windows volumes have mount points
such as `C:\` and folders such as `C:\Users\me`, so no volume ever matches and the free space part of
the bar stays hidden on Windows. Found while reviewing the DESK-027 code on 2026-11-11.

## Steps to reproduce

1. On Windows, open `C:\Users\<you>` in the Explorer.
2. The status bar shows the item counts but no "free" part on the right.

## Acceptance Criteria

- [x] `volumeForPath` matches Windows drive roots and folder mount points on whole segments
- [x] `statusBar.test.ts` covers Windows paths

## Fix

`isUnder` in `statusBar.ts` trims either separator from the mount point and accepts the path when the
character after it is the end of the path, `/` or `\`, so `C:\Users\me` sits on `C:\` and
`C:\mnt\database` does not sit on a `C:\mnt\data` mount. "matches Windows drives and folder mount
points" in `statusBar.test.ts` covers it; `bun test
packages/interface/src/routes/explorer/components/statusBar.test.ts` passes 7 tests. The test file was
not in the CI list, so it is added to `.github/workflows/ci.yml` (Linux and, through
`scripts/ci-frontend-tests.sh`, Windows). Not run on a Windows machine.
