---
id: DESK-046
title: Cut files are dimmed until pasted
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer, clipboard]
last_updated: 2026-11-03
---

## Description

After Cut, nothing showed which files the next paste would move. Dolphin, Nautilus and Windows
Explorer draw them faded.

## Acceptance Criteria

- [x] Cut items render at half opacity in grid, list, column and media views
- [x] The dimming clears after paste or a new copy

## Evidence

- `useIsCut(path)` in `useClipboard.ts` reads the clipboard store; FileCard, TableRow, ColumnItem and MediaViewItem apply `opacity-50`.
- Web UI, 2026-11-03: after Ctrl+X on `doc1.txt` its row had opacity 0.5 and its neighbours 1.
- Desktop debug build, 2026-11-03: `target/debug/WingDrive` under Xvfb with a private D-Bus session, isolated data dir, instance, XDG_DATA_HOME and XDG_RUNTIME_DIR, driven through WebKitWebDriver with real key events. Ctrl+X on `test.txt` dimmed it to 0.5; Ctrl+C on another file cleared it.
