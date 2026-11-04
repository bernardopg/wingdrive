---
id: DESK-041
title: Escape does not close View Settings; empty search keeps hiding the view buttons
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard, bug]
last_updated: 2026-11-03
---

## Description

Seen during the packaged runtime check on 2026-11-02:

- The View Settings popover stays open on Escape; only a click outside or on its button closes it.
- After "Clear Search", the search field stays expanded and empty, and the tag, Views and View
  Settings buttons stay hidden until the field loses focus.

## Acceptance Criteria

- [x] Escape closes the View Settings popover
- [x] Clearing the search collapses the field and brings the toolbar buttons back

## Evidence

- View Settings listens for Escape (capture phase) while open, closes and returns focus to its button.
- The search field collapses when its value is cleared from outside the field (Clear Search); cleared from
  inside, it stays open for a new query.
- Desktop debug build, 2026-11-03 (Xvfb, private D-Bus, isolated daemon, WebKitWebDriver with real key and pointer events): Escape closed View Settings. After searching `doc`, the toolbar showed only the field; a click on
  Clear Search collapsed it and brought back Search, Tag Mode, Views and View Settings.
