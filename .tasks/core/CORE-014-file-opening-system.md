---
id: CORE-014
title: Cross-Platform File Opening Backend
status: In Progress
assignee: jamiepine
priority: High
tags: [core, platform, file-operations]
whitepaper: DESIGN-open-with.md
last_updated: 2026-10-08
related_tasks: [EXPL-004, DESK-004]
---

## Description

Implement the backend infrastructure for opening files with default and specific applications across macOS, Windows, and Linux. This ports v1's sophisticated platform-specific implementation to v2's architecture.

## Implementation Notes

Create platform-specific crates following v1's proven architecture:
- `apps/tauri/crates/file-opening/` - Shared types and traits
- `apps/tauri/crates/file-opening-macos/` - Swift via FFI using NSWorkspace APIs
- `apps/tauri/crates/file-opening-windows/` - COM Shell APIs (SHAssocEnumHandlers)
- `apps/tauri/crates/file-opening-linux/` - GTK/GIO with content type detection

Each platform implementation must:
1. Query OS for applications that can open a file
2. Return intersection of compatible apps for multi-file selection
3. Open file with default application
4. Open file(s) with specific application

See `DESIGN-open-with.md` for complete architecture details.

## Acceptance Criteria

Audit 2026-10-08: the code for all three platforms exists. macOS and Windows boxes stay open because
they can only be verified on those systems; the code locations are listed so the check is quick there.

- [x] Shared `file-opening` crate with `FileOpener` trait and types
      (`apps/tauri/crates/file-opening/src/lib.rs:7` `OpenWithApp`, `:30` `OpenResult`, `:39` `FileOpener`;
      types live in `lib.rs`, no separate `types.rs`)
- [ ] macOS implementation using Swift FFI + NSWorkspace (needs a Mac to verify)
  - [ ] Query apps using `urlsForApplications(toOpen:)` API (code: `file-opening-macos/src-swift/FileOpening.swift:82`)
  - [ ] Filter to `/Applications/` directory (code: `FileOpening.swift:93`, also `/System/Applications` and `~/Applications`)
  - [ ] Open with default via NSWorkspace (code: `FileOpening.swift:118`)
  - [ ] Open with specific app by bundle ID (code: `FileOpening.swift:142`, `:175` for several files)
- [ ] Windows implementation using COM Shell APIs (needs Windows to verify)
  - [ ] Query apps using `SHAssocEnumHandlers` (code: `file-opening-windows/src/lib.rs:132`)
  - [ ] Thread-local COM initialization (code: `file-opening-windows/src/lib.rs:16`)
  - [ ] Open with default via ShellExecute (code: `file-opening-windows/src/lib.rs:53`)
  - [ ] Open with specific app via IAssocHandler (code: `file-opening-windows/src/lib.rs:83`, `:116`)
- [x] Linux implementation (done with the XDG specifications directly instead of GTK/GIO, see DESK-004)
  - [x] Content type detection: shared-mime-info globs first like GIO, then content sniffing through
        `xdg-mime query filetype` / `file --mime-type` (`file-opening-linux/src/mime_apps.rs:196`, `:223`, `:216`)
  - [x] Query apps from desktop entries + `mimeapps.list` defaults/added/removed associations with MIME subclass
        lineage, the data `AppInfo::recommended_for_type` reads (`mime_apps.rs:90`, `:144`, `:336`)
  - [x] Open with default via `xdg-open` with a clean host environment (`file-opening-linux/src/host_env.rs:59`)
  - [x] Open with specific app by desktop entry ID, Exec field codes expanded without a shell
        (`file-opening-linux/src/lib.rs:62`, `desktop_entry.rs:185`)
- [x] Tauri commands registered (`apps/tauri/src-tauri/src/main.rs:2433-2437`):
  - [x] `get_apps_for_paths(paths)` - returns Vec<OpenWithApp> (`apps/tauri/src-tauri/src/file_opening.rs:42`)
  - [x] `open_path_default(path)` - returns OpenResult (`file_opening.rs:56`)
  - [x] `open_path_with_app(path, app_id)` - returns OpenResult (`file_opening.rs:67`)
  - [x] `open_paths_with_app(paths, app_id)` - returns Vec<OpenResult> (`file_opening.rs:79`)
- [x] Intersection logic for multi-file selections works correctly (`file-opening/src/lib.rs:47`; unit test
      `intersection_keeps_first_files_order_and_shared_apps_only` at `:132`)
- [x] Error handling returns proper OpenResult variants (Linux: `FileNotFound` `file-opening-linux/src/lib.rs:44`,
      `:68`; `AppNotFound` `:74`; `PlatformError` `:51`, `:82`; the UI toasts each variant in
      `packages/interface/src/hooks/useOpenWith.ts` `handleOpenResult`. Linux never reports `PermissionDenied`:
      an unreadable file surfaces as the app's own error)
- [x] All commands are async and non-blocking (`file_opening.rs:33` runs every opener call on the blocking pool)

## Interface (2026-10-08)

- Double-click opens through the default handler in every view (`useOpenFile`, e.g.
  `packages/interface/src/routes/explorer/views/GridView/FileCard.tsx:61`).
- Enter opens the selection on Linux and Windows, Cmd+Down on macOS (`explorer.openSelection` in
  `packages/interface/src/util/keybinds/registry.ts:21`); Cmd/Ctrl+O and the context menu's Open now open
  every selected file (`useOpenFiles` in `packages/interface/src/routes/explorer/hooks/useOpenFile.ts`), up to
  20 at once, skipping folders when files are selected too.
- Open With / Always Open With show app icons on Linux (`file-opening-linux/src/icon_theme.rs`), in the native
  menu through `IconMenuItem` (`apps/tauri/src/contextMenu.ts`) and in the web menu.

## Implementation Files

To be created:
- `apps/tauri/crates/file-opening/src/lib.rs`
- `apps/tauri/crates/file-opening/src/types.rs`
- `apps/tauri/crates/file-opening-macos/src/lib.rs`
- `apps/tauri/crates/file-opening-macos/src-swift/FileOpening.swift`
- `apps/tauri/crates/file-opening-windows/src/lib.rs`
- `apps/tauri/crates/file-opening-linux/src/lib.rs`
- `apps/tauri/src-tauri/src/commands/file_opening.rs`

To be modified:
- `apps/tauri/src-tauri/src/main.rs` (register commands and service)
- `apps/tauri/src-tauri/Cargo.toml` (add dependencies)

## Reference Implementation

v1 implementation can be found at:
- `~/Projects/wingdrive_v1/apps/desktop/src-tauri/src/file.rs`
- `~/Projects/wingdrive_v1/apps/desktop/crates/macos/src-swift/files.swift`
- `~/Projects/wingdrive_v1/apps/desktop/crates/windows/src/lib.rs`
- `~/Projects/wingdrive_v1/apps/desktop/crates/linux/src/app_info.rs`

## Testing

- Unit tests for intersection logic
- Platform-specific tests with mock file system
- Manual testing on macOS, Windows, Linux
- Test edge cases: no apps available, permission denied, file not found
