---
id: DESK-032
title: User-defined context menu actions
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
tags: [desktop, explorer, linux, extensibility]
last_updated: 2026-10-27
---

## Description

Thunar custom actions, Dolphin service menus and Nautilus scripts let users add commands to the
context menu, filtered by file type. WingDrive has no such hook.

## Implementation Steps

1. Read Dolphin-compatible service menus (`.desktop` files with `Actions=` under
   `~/.local/share/kio/servicemenus`) and a WingDrive config of command, label and MIME filter.
2. Show matching actions in the context menu and run them with the selected paths.

## Acceptance Criteria

- [ ] A user action appears for matching files and runs with the selection
