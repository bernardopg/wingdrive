---
id: DESK-098
title: Upload to Google Photos is missing from the menu of a multi-file selection
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M3
sprint: S06
tags: [explorer, google-photos, context-menu, bug]
last_updated: 2026-11-22
---

## Description

Found on 2026-11-16 while recording the INTG-003 demo. With three local photos selected (Ctrl+A), the Explorer context menu has no "Upload to Google Photos"; with one photo it does. The single-file case also never worked before this date: `canUploadToPhotos` checked the extension on `File.name`, which is the stem without extension (fixed in the INTG-003 pull request).

## Implementation Steps

- [x] Find why `targetFiles` or `canUploadToPhotos` rejects a multi-selection of local photos.
- [x] Regression test with several selected `File` objects shaped like the daemon sends them.

## Acceptance Criteria

- [x] Selecting several local photos shows Upload to Google Photos and uploads all of them.

## Implementation Notes

- Root cause: `useFileContextMenu` computed `targetFiles` from the click-time `selected` prop. When multi-selection state was not reflected by that flag, the menu fell back to the clicked file instead of the full selection. Fixed by deriving selection membership from `selectedFiles` (`fileInSelection`) so any selected file acts on the whole selection.
- Also aligned `openTargets` with the same selection membership check for consistency.
- Added regression coverage in `useGooglePhotosUpload.test.ts` for several daemon-shaped files (stem name + extension field), including 3-photo multi-select and mixed case extensions.

## Evidence

- `cd packages/interface && bun test src/routes/explorer/hooks/useGooglePhotosUpload.test.ts` - 3 pass, 0 fail.
- `cd packages/interface && bun test src/routes/explorer/hooks/useGooglePhotosUpload.test.ts src/routes/explorer/FileContextMenuProvider.test.tsx` - 3 pass, 1 unrelated env error (`@phosphor-icons/react` resolution from built primitives dist in incomplete worktree deps).
- `bun run typecheck` - no interface errors; fails in `@wingdrive/mobile` due missing React Native deps in this worktree environment.
