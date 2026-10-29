---
id: DESK-032
title: User-defined context menu actions
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer, linux, extensibility]
sprint: S04
last_updated: 2026-10-28
---

## Description

Thunar custom actions, Dolphin service menus and Nautilus scripts let users add commands to the
context menu, filtered by file type. WingDrive has no such hook.

## Implementation Steps

1. Read Dolphin service menus (`Type=Service` desktop files with `Actions=` under
   `kio/servicemenus` and the older `kservices5/ServiceMenus` in every XDG data directory), plus
   `wingdrive/actions` in the same format for WingDrive-only actions. As in KIO, a file in the
   user's data directory only counts when it is executable.
2. Match `MimeType` (exact, `image/*`, `all/all`, `all/allfiles`, `inode/directory`) against every
   selected item's MIME lineage, and honour `X-KDE-RequiredNumberOfUrls`, `X-KDE-MinNumberOfUrls`
   and `X-KDE-MaxNumberOfUrls`, `TryExec` and `Hidden`.
3. Show matching actions in the file context menu, one submenu per `X-KDE-Submenu` and an
   "Actions" submenu for the rest, and run them with the selected paths through the `Exec` field
   codes, without a shell.

## Acceptance Criteria

- [x] A user action appears for matching files and runs with the selection
- [ ] Verified on the Linux desktop build

## Evidence

- `apps/tauri/crates/file-opening-linux/src/service_menus.rs`: parsing, localized names and
  submenus, MIME and count filters, the executable rule, user files hiding system copies, and
  command lines; 8 unit tests, now run in CI with the rest of the crate.
- Run against the author's real menus (`~/.local/share/kio/servicemenus`, 11 files): a PDF gets
  12 actions and a folder 14, matching their `MimeType`; `jpg2png` and `extract-archives` stay
  hidden for both. Nothing was executed.
- `run_context_action` looks the action up again and refuses one that no longer applies.
- Frontend: `useContextActions` and `contextActionMenus` (covered by `useContextActions.test.ts`).
- Open: run an action from the menu in the Linux desktop build.
