---
id: EXPL-003
title: File Operations UI
status: Done
assignee: jamiepine
parent: EXPL-000
priority: High
milestone: M1
sprint: S01
tags: [explorer, file-operations]
whitepaper: N/A
last_updated: 2026-10-02
---

## Description

Implement UI for core file operations: copy, move, delete, rename. Integrates with backend jobs and shows progress.

## Implementation Notes

- Use useLibraryMutation for all operations
- Show progress toast for long operations
- Subscribe to job progress events
- Handle errors gracefully with user feedback
- Confirmation dialogs for destructive operations

## Acceptance Criteria

- [x] Copy files via context menu or Cmd+C
- [x] Move files via drag and drop
- [x] Delete with confirmation dialog
- [x] Rename with inline editing
- [x] Duplicate files (Cmd+D, "name copy" suffix, AutoModifyName conflict resolution)
- [x] Create new folders
- [x] Progress indicator for long operations (FileOperationModal)
- [x] Error handling with user-friendly messages
- [x] Undo for safe operations

## Verification (2026-10-02)

Native tests validate identity-checked undo and refusal to overwrite changed destinations. Supported undo: rename, same-volume move without overwrite, and creation of an empty local folder. Cross-volume moves, overwrites and deletion are excluded. Remote paths are not registered in the local undo journal.
