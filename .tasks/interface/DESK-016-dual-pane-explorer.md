---
id: DESK-016
title: Dual-pane Explorer
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, linux]
last_updated: 2026-11-05
sprint: S05
---

## Description

Dolphin's split view (F3) shows two folders side by side, which makes moving
and comparing files fast. WingDrive has no split view. This is the larger of
the two "two places at once" options; DESK-015 (a new window) covers most of
the need with far less work, so decide on this one after DESK-015 ships.

## Implementation Steps

1. Decide the model: two independent Explorer contexts in one window, each
   with its own path, history, selection and view settings.
2. Toggle with F3 and a toolbar button; the focused pane receives keyboard
   shortcuts and toolbar actions.
3. Drag and drop and copy/paste between panes use the existing file
   operations; "Copy/Move to other pane" goes in the context menu.
4. Persist the split state per tab.

## Design notes (2026-10-28)

Every tab owns a router (`TabManagerContext.tsx`), and `TabView` renders only the active tab's
router. `ShellLayout` (sidebar, Inspector, TopBar, `SelectionProvider`, `ExplorerProvider`) is the
layout route inside that router, and `ExplorerProvider` reads its path from `useLocation()` and keys
its state by `activeTabId`. Two panes therefore cannot be two `ExplorerProvider`s under one router.

The cheapest model is a pane per tab: the second pane renders another tab's router, and the shell
chrome moves out of the router so both panes share one sidebar, Inspector and TopBar, which follow
the focused pane. That move touches every view that registers TopBar buttons through the portal and
every hook that reads `activeTabId`, so plan it as its own sprint item (several days) and do it after
DESK-015 is validated, as the description says.

## Acceptance Criteria

- [x] F3 toggles a second pane with its own navigation
- [x] Shortcuts act on the focused pane only
- [x] Copy and move between panes work by drag and by context menu
- [x] Verified on the packaged Linux build (author, `wingdrive-bin` 3.0.0, 2026-11-05)

## Implementation (2026-11-04)

A pane per tab, as the design notes proposed, without moving the shell out of the router:

- `TabManagerContext` keeps a split per tab (`splits`): a hidden second tab with its own router, path,
  history and explorer state, persisted with the tabs. `toggleSplit` opens it in the owner's folder and
  view; closing keeps it, so F3 reopens the same folder. Closing the tab drops its pane.
- `TabView` renders both routers side by side. Each `Pane` scopes `useTabManager` so `activeTabId` is
  that pane's tab, and provides `PaneContext` (`pane`, `split`, `focused`). A pointer down or focus
  inside a pane makes it the focused pane; the sidebar is marked `data-pane-neutral`.
- `ShellLayout` shows the sidebar only in the left pane and the Inspector only in the right one; the
  tab bar stays on the left. The focused pane gets an accent ring.
- `useKeybind` registers only in the focused pane, and the views' own `keydown` listeners (grid, list,
  column, media, explorer keyboard, path bar) return early in the unfocused pane.
- The sidebar navigates the focused pane (`useSidebarNavigate`).
- "Copy to Other Pane" and "Move to Other Pane" in the file context menu; each pane is a drop target for
  the folder it shows (folders inside it still win), and dropping files on the pane that already holds
  them does nothing. F3 and a toolbar button toggle the split.

## Evidence

- Desktop debug build, 2026-11-04 (Xvfb, private D-Bus, isolated daemon, WebKitWebDriver with real key and
  pointer events, xdotool for drags):
  - F3 opened a second pane in the same folder; the left kept the sidebar, the right the Inspector.
  - Opening `docs` in the right pane left the left pane in the location root.
  - Ctrl+A in the focused left pane selected its 3 items; the right pane kept 0.
  - With the right pane focused, sidebar Overview and then the `loc` location navigated the right pane only.
  - Right pane in `photos`: "Copy to Other Pane" on `readme.txt` in the left opened the copy dialog to
    `photos`; after confirming, `photos/readme.txt` existed and showed in the right pane.
  - Dragging `notes.md` and then `annual-report-2026.txt` from the left pane onto the right pane opened the
    dialog with destination `photos`; the files were copied there.
  - Relaunching the app restored both panes and their folders. F3 closed the split, and F3 again reopened it
    in `photos`.
- Open: the dialog of a drop on the same device defaults to Copy instead of Move, also for a drop on a
  folder inside one pane, so it predates this task (DESK-050). Verify on the packaged build.
- 2026-11-05: the author reports the dual pane working in daily use of the installed AUR package
  (`wingdrive-bin` 3.0.0). The Copy default on same-device drops was fixed separately (DESK-050).
