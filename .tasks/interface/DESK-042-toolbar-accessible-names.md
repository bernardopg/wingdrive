---
id: DESK-042
title: Icon-only Explorer toolbar buttons have no accessible name
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S04
tags: [desktop, explorer, accessibility]
last_updated: 2026-11-03
---

## Description

The packaged runtime check on 2026-11-02 listed the Explorer toolbar buttons: the sidebar
toggle, back, forward, search, tag mode, Inspector and the overflow menu have neither
`aria-label` nor `title`, so screen readers announce them as "button". View Settings and
Show Terminal Panel already have a title.

## Acceptance Criteria

- [x] Every icon-only toolbar button has a translated accessible name (en, pt)

## Evidence

- `CircleButton` uses its `title` as `aria-label` when it has no label and no text, so every titled icon
  button gets a name. Explorer toolbar: Sidebar, Back, Forward, Search, Tag Mode, Inspector and More
  (overflow) got translated titles (`topBar.*`, en and pt).
- Desktop debug build, 2026-11-03 (Xvfb, private D-Bus, isolated daemon, WebKitWebDriver with real key and pointer events): every button in the Explorer toolbar had a name: Sidebar, Back, Forward, Device (image alt), Location
  is indexed, Search, Tag Mode, Views, View Settings, Show Terminal Panel, Inspector, More. The Inspector's
  tab icons are outside this task and still have no name.
