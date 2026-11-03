---
id: DESK-040
title: Context menu lost its first items in a short window
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, explorer, context-menu, bug]
last_updated: 2026-11-02
---

## Description

The web context menu used on Linux (`WebContextMenuContext.tsx`) had no height limit. A
file menu is about 560 px tall, so in a window around 600 px high it was pushed above the
top edge and Quick Look, Open, Open With and Actions could not be reached.

Found during the packaged runtime check of the S04 tasks.

## Acceptance Criteria

- [x] The menu and its submenus fit the window and scroll when they are taller
- [x] Verified on the packaged Linux build

## Evidence

- Fix: menu and submenu content take `max-h-[var(--radix-dropdown-menu-content-available-height)]`
  with `overflow-y-auto`.
- Packaged build, 2026-11-02, window 1400x520: the file menu sat between y=8 and y=301 inside the
  520 px viewport, scrolled, and its first item (Quick Look) was visible at y=13.
