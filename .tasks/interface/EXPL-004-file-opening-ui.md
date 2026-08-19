---
id: EXPL-004
title: File Opening UI Integration
status: In Progress
assignee: jamiepine
parent: EXPL-000
priority: High
tags: [explorer, file-operations, ui]
whitepaper: DESIGN-open-with.md
last_updated: 2026-08-19
related_tasks: [CORE-014]
---

## Description

Integrate file opening functionality into Explorer UI. When users double-click a file, it should open with the system's default application. The context menu should provide "Open" and "Open With" options showing available applications.

## Dependencies

Requires CORE-014 (backend file opening system) to be completed first.

## Implementation Notes

- Extend platform interface with file opening methods
- Create `useOpenWith` React hook for querying apps and opening files
- Update context menu to show "Open With" submenu
- Wire up double-click handlers in all view components
- Handle multi-file selection (show intersection of compatible apps)
- Use toast notifications for errors

Currently there's a TODO at `packages/interface/src/components/Explorer/hooks/useFileContextMenu.ts:82-83` that needs to be replaced with actual implementation.

See `DESIGN-open-with.md` for complete architecture details.

## Acceptance Criteria

- [x] Platform interface extended with:
- [x] `getAppsForPaths(paths: string[]): Promise<OpenWithApp[]>`
- [x] `openPathDefault(path: string): Promise<OpenResult>`
- [x] `openPathWithApp(path, appId): Promise<OpenResult>`
- [x] `openPathsWithApp(paths, appId): Promise<OpenResult[]>`
- [x] `useOpenWith` hook created with:
- [x] React Query integration fetching apps
- [x] `openWithDefault(path)` function
- [x] `openWithApp(path, appId)` function
- [x] `openMultipleWithApp(paths, appId)` function
- [x] Proper error handling toast notifications
- [x] Context menu integration:
  - [x] "Open" menu item works for files and folders
  - [x] "Open With" submenu appears for files
  - [x] Shows list of compatible applications
  - [x] For multi-select, only shows apps that can open ALL files
  - [x] Apps are sorted alphabetically
- [x] Double-click handlers updated in:
  - [x] GridView (`FileCard.tsx`)
  - [x] ListView (`TableRow.tsx`)
  - [x] ColumnView (`Column.tsx`)
  - [x] MediaView (if applicable)
- [x] Open keybind (Cmd+O) wired in `useExplorerKeyboard.ts`
- [x] Error handling:
  - [x] File not found → toast error
  - [x] App not found → toast error
  - [x] Permission denied → toast error
  - [x] Platform errors → toast with message
- [x] Loading states shown while querying apps
- [x] TODO at `useFileContextMenu.ts:82-83` is removed

## Implementation Files

To be created:
- `packages/interface/src/hooks/useOpenWith.ts`

To be modified:
- `apps/tauri/src/platform.ts` (extend interface)
- `packages/interface/src/components/Explorer/hooks/useFileContextMenu.ts`
- `packages/interface/src/components/Explorer/views/GridView/FileCard.tsx`
- `packages/interface/src/components/Explorer/views/ListView/TableRow.tsx`
- `packages/interface/src/components/Explorer/views/ColumnView/Column.tsx`

## User Experience

**Before:**
- Double-clicking files does nothing
- No "Open" or "Open With" in context menu
- Users can only use "Quick Preview"

**After:**
- Double-clicking files opens them in default app
- Double-clicking folders still navigates (unchanged)
- Context menu has "Open" option (⌘O)
- Context menu has "Open With" submenu showing available apps
- Multi-select shows only apps compatible with all selected files
- Proper error messages if opening fails

## Testing

- Test double-click on various file types (.txt, .pdf, .jpg, .mp4)
- Test double-click on folders still navigates
- Test "Open" in context menu
- Test "Open With" shows correct apps
- Test multi-select intersection logic
- Test error cases: missing file, no apps, permission denied
- Test on all platforms: macOS, Windows, Linux
