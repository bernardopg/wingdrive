---
id: DESK-041
title: Escape does not close View Settings; empty search keeps hiding the view buttons
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, keyboard, bug]
last_updated: 2026-11-02
---

## Description

Seen during the packaged runtime check on 2026-11-02:

- The View Settings popover stays open on Escape; only a click outside or on its button closes it.
- After "Clear Search", the search field stays expanded and empty, and the tag, Views and View
  Settings buttons stay hidden until the field loses focus.

## Acceptance Criteria

- [ ] Escape closes the View Settings popover
- [ ] Clearing the search collapses the field and brings the toolbar buttons back
