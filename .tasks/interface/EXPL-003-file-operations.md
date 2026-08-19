---
id: EXPL-003
title: File Operations UI
status: In Progress
assignee: jamiepine
parent: EXPL-000
priority: High
tags: [explorer, file-operations]
whitepaper: N/A
last_updated: 2026-08-19
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
- [ ] Undo for safe operations
