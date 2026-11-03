---
id: DESK-042
title: Icon-only Explorer toolbar buttons have no accessible name
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, explorer, accessibility]
last_updated: 2026-11-02
---

## Description

The packaged runtime check on 2026-11-02 listed the Explorer toolbar buttons: the sidebar
toggle, back, forward, search, tag mode, Inspector and the overflow menu have neither
`aria-label` nor `title`, so screen readers announce them as "button". View Settings and
Show Terminal Panel already have a title.

## Acceptance Criteria

- [ ] Every icon-only toolbar button has a translated accessible name (en, pt)
