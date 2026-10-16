---
id: FORK-009
title: Contribute WingDrive fixes back to upstream Spacedrive
status: In Progress
assignee: bernardopg
parent: FORK-002
priority: Medium
milestone: M2
sprint: S05
tags: [fork, upstream, contribution]
last_updated: 2026-10-16
---

## Description

The 2026-10-16 audit (`docs/design/UPSTREAM_CONTRIBUTIONS.md`) found 17 WingDrive fixes whose bug still exists in
upstream `main` (`4d41a379f`), with no matching upstream issue or PR. Each
contribution is a branch off `upstream/main` in a fork of
`spacedriveapp/spacedrive`, with upstream names (`sd_core`, `SdPath`,
`.sdlibrary`), no WingDrive task IDs or i18n keys, and no AI co-author trailers.
Opening each upstream PR needs the owner's go-ahead.

## Ranked contributions

| # | WingDrive commits | Fix | Effort | Status |
|---|---|---|---|---|
| 1 | `23b09f2bd` | Sync catch-up applies buffered changes before Ready | S | To Do |
| 2 | `a4ceb17ab` | Library watcher does not reopen a just-created library | S | To Do |
| 3 | `ccf7e046b` (eject part) | Eject refuses system, `/` and `/home` mounts | S | To Do |
| 4 | `361332d93` | Secure delete refuses copy-on-write filesystems | M | To Do |
| 5 | `b55328b8c` | `Vec<T>` generates `T[]`, not the tuple `[T]` | M | To Do |
| 6 | `d5b619da2`, `7ccb3912f` | Daemon lock ignores zombies; stop waits; app close stops its daemon | M | To Do |
| 7 | `6696b48b4` | `mktemp` in adapters, `__proto__` key, `codesign` via shell | S | To Do |
| 8 | several | Windows reveal args, hook in `.map()`, refused transfer message, alias compare | S | To Do |

## Acceptance Criteria

- [x] Fork of `spacedriveapp/spacedrive` under `bernardopg`
- [x] Items 1 to 3 prepared as branches with a test each, then opened upstream
  (spacedriveapp/spacedrive#3160, #3161, #3162, opened 2026-10-16; CI awaits maintainer approval)
- [ ] Items 4 to 8 prepared or dropped with a reason
