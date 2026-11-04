---
id: DESK-044
title: Typeahead scrolls to the match, and T no longer opens tag mode
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S04
tags: [desktop, explorer, keyboard, bug]
last_updated: 2026-11-03
---

## Description

Typing the first letters of a name selected the matching file but did not scroll to it when the
grid or list had not rendered that row yet (virtualized rows have no element to scroll). A bare
`T` also entered tag assignment mode, so typing a name that starts with T switched modes.

Reported by the author on 2026-11-03.

## Acceptance Criteria

- [x] Typeahead brings a match far down the listing into view
- [x] Typing T selects files starting with T and stays out of tag mode
- [x] Tag mode keeps a shortcut

## Evidence

- Typeahead uses `scrollToRevealed`, the reveal event the virtualized views already handle, instead of `scrollIntoView` on an element that may not exist.
- Tag mode moved to Alt+G (`explorer.enterTagMode`). Alt+T was the first choice but the toaster already uses it to focus notifications.
- Web UI on an isolated daemon, 2026-11-03: 31 files, `file1.txt` selected, typing `tes` selected `test.txt` and it sat inside the 398 px viewport (top 229); no tag mode.
- Desktop debug build, 2026-11-03: `target/debug/WingDrive` under Xvfb with a private D-Bus session, isolated data dir, instance, XDG_DATA_HOME and XDG_RUNTIME_DIR, driven through WebKitWebDriver with real key events. 62 files: typing `tes` selected `test.txt`, the last item, and scrolled it into view (top 581 of 750); Alt+G opened Tag Mode, Escape closed it.
