---
id: TAURI-013
title: Linux desktop data-safety stabilization
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M1
sprint: S01
tags: [tauri, linux, safety, interface, regression]
last_updated: 2026-10-05
---

## Description

Fix the data-safety items from the desktop UI audit. File commands must act on what you see, and partial results must never look like success. Features that do nothing must stop pretending they work. Preserve existing data and contracts and avoid new runtime dependencies. Linux is the target.

The rest of the audit (startup recovery, auxiliary windows, search parity, clipboard/drag/pairing) moved to TAURI-014 so this slice fits S01.

## Implementation Steps

- [x] Unify active collections, selection reconciliation, virtual-entity capabilities and operational directory
- [x] Preserve partial copy results and represent cancelled, failed and skipped outcomes
- [x] Remove unsupported encryption, sync, language and experimental promises; correct volume intersection
- [x] Add targeted tests for selection, partial outcomes and hidden features

## Acceptance Criteria

- [x] File commands operate on the current displayed selection, never virtual entities or stale paths
- [x] Column operations target the active directory and exposed views honor their source
- [x] Partial copy/move/duplicate outcomes remain visible and cannot be reported as integral success
- [x] Visible preferences and redundancy assertions have real, testable behavior
- [x] Experimental features are unavailable or truthful; Linux capabilities are exposed accurately
- [x] Frontend tests, typecheck/build and relevant Rust validation pass, with limitations recorded

## Validation

Baseline: main b17f989a2; clean worktree; bun run typecheck passed.

### What changed

- Selection: the explorer keeps one displayed collection (`currentFiles`), stamped with the tab, route, view and mode that published it. The selection is rebuilt from that collection whenever it changes, so entries that were deleted, renamed or are no longer shown drop out and commands receive the latest File objects. Keyboard select-all and typeahead read the same collection instead of a second directory query.
- Virtual entries: `selectionCapabilities` disables copy, cut, rename, duplicate and delete whenever a device, volume or location card is selected. The keyboard, context menu, native menu items, drag-and-drop and the delete and duplicate hooks all check it, so a location card can no longer send its root path to `files.delete`.
- Operational directory: New Folder, Paste and external drops write to `operationalPath`, which is the active column's folder in column view and null in search, recents, tags, filters and device listings.
- Views honor their source: Media and Size use the explorer collection for recents, tags, filters, search and device listings; Column renders a single results column for those sources; a persisted Knowledge view falls back to grid in production.
- Partial outcomes: `JobOutput::FileCopy` now reports `failed_count`, `skipped_count` and up to 20 error messages, and `FileMove` reports `skipped_count` and errors. Skipped items no longer count as copied. One unresolvable source, a unique-name failure or an Abort conflict no longer discard items already copied. The copy dialog, duplicate and delete read the counts and show success, partial, failed, cancelled or still running. Older job history without the new fields still deserializes.
- Truthful settings: Library settings show only volume auto-tracking (system volume tracking now honors its setting); thumbnail, AI tagging, sync and encryption switches were removed because nothing reads them. The language picker, telemetry switch, daemon log level and job logging switch were removed or replaced with accurate text. The network storage category and the `/search` "coming soon" route were removed.
- Volume intersection: `on_volumes` now requires every listed volume, so Redundancy "shared" lists content present on both volumes instead of either one.

### Evidence (2026-10-05)

- `cargo test -p wing-core --lib`: `on_volumes_filter_requires_every_volume`, `file_copy_output_from_older_builds_still_deserializes`, `file_move_output_from_older_builds_still_deserializes` pass.
- `cargo test -p wing-core --test copy_action_test`: 8 passed, including `test_copy_reports_skipped_items` (1 copied, 1 skipped, existing file kept).
- `bun test`: `fileCapabilities` (11), `fileOperationOutcome` (13), `effectiveViewMode` (2), `settingsPages` (4) pass; added to the CI frontend step.
- `bun run typecheck` passes after regenerating `packages/ts-client/src/generated/types.ts`.
- Runtime, isolated `wing-server --instance t013` with the web build in Chromium:
  - Device view: selecting the Home location card and pressing Delete opens no dialog; its context menu offers only Quick Look and Open.
  - A selected file deleted on disk leaves the listing and a following Delete targets nothing; other files untouched.
  - Copying `a.txt` and `b.txt` into a folder that has `a.txt` with Skip shows "Finished with Problems: Copied 1 item, 1 skipped"; the existing file keeps its content.
  - Column view with a file selected in the second column: Ctrl+Shift+N creates the folder in that column's folder, not the root.
  - Recents in Media view shows the recent image instead of "No location selected".

### Limitations

- Settings pages were checked by rendering tests; the web build has no Settings window, so the native window still needs a click-through (TAURI-014).
- Redundancy compare was verified with the SQL unit test; no second volume with shared content was available at runtime.
- A move that fails partway leaves the clipboard cut in place so the user can retry; the Jobs screen still shows per-file status only.
