---
id: DESK-047
title: Expand and collapse all in the sidebar folder tree
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, sidebar, folder-tree]
last_updated: 2026-11-03
---

## Description

The Folders section of the sidebar (DESK-029) had no way to fold every branch at once or open
many at once.

## Acceptance Criteria

- [x] A collapse button closes every folder
- [x] An expand button opens the next level of the folders on screen without listing the whole disk

## Evidence

- Two buttons beside the Folders header. Collapse clears the expanded set. Expand opens the shallowest closed folders on screen, at most 50 per click, because a recursive expand of `/` would list every folder on the disk.
- Web UI, 2026-11-03: collapse left 1 row; expand opened 1 folder level (17 rows), the next click 16 more folders (1234 rows).
- Desktop check found two faults, both fixed:
  - Expand never reopened Home after Computer had shown `/home/bitter` and closed it: the nested copy
    removed the path from the set of visible folders. Visible folders are now counted per path.
  - Each open folder held its own daemon event subscription, so a few expand clicks hit the daemon's
    100-connection limit (`Connection limit reached (100)`) and the app fell back to "Starting WingDrive".
    `useNormalizedQuery` has a `live` option; tree folders use `live: false` and refresh on reopen, on window
    focus, and after the explorer's own file operations. Test in `normalizedQuerySubscription.test.tsx`.
- Desktop debug build, 2026-11-03: `target/debug/WingDrive` under Xvfb with a private D-Bus session, isolated data dir, instance, XDG_DATA_HOME and XDG_RUNTIME_DIR, driven through WebKitWebDriver with real key events. Collapse, then four expands: 0, 2, 75, 99 and 115 open folders; the app held 34 daemon connections
  at every step and the daemon logged no new `Connection limit` line. Home and Computer both reopen.
