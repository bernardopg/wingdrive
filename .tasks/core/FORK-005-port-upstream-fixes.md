---
id: FORK-005
title: Relicense to Apache-2.0 and port selected upstream fixes
status: In Progress
assignee: bernardopg
parent: FORK-002
priority: High
milestone: M1
sprint: S02
tags: [fork, upstream, license, port]
last_updated: 2026-10-05
---

## Description

Upstream Spacedrive is active: 243 commits since the WingDrive fork point
`6dfeccf21` (2026-07-28), almost all by one author. Upstream replaced the
locations/entries model with source stores, arenas and a new library schema,
removed Spacebot and started a GPUI client. WingDrive keeps its own model, so
upstream is never merged wholesale. Fixes that do not depend on the new model
are ported commit by commit, keeping authorship and `cherry picked from`
trailers. Anything WingDrive already fixed on its own keeps the WingDrive
version.

Upstream relicensed to Apache-2.0 in `bcc124765`, whose only parent is the
fork point, so WingDrive adopts Apache-2.0 too (branch
`chore/relicense-apache-2.0`).

## Acceptance Criteria

- [x] LICENSE, NOTICE.md, README, CONTRIBUTING, crate/package fields, AUR
      PKGBUILD and About screen say Apache-2.0
- [x] Every upstream commit classified (port, already fixed, not applicable)
- [x] Portable fixes applied on `port/upstream-fixes` with authorship kept
- [ ] `cargo clippy --workspace -D warnings`, `bun run typecheck` and affected
      integration tests pass
- [ ] Both branches merged to `main`

## Validation

- `cargo check -p wing-core -p wing-cli --lib --bins` clean.
- Integration tests on `port/upstream-fixes`: `event_filtering_test` 9/9,
  `file_transfer_with_restart_test` 3/3 (2 ignored), `library_test` 5/5,
  `sync_backfill_test` 5/5, `sync_event_log_test` 12/12.
- `sync_metrics_test`: 2 failures (`test_metrics_broadcast_counting`,
  `test_metrics_data_volume`). Not a regression: on `main` the same file fails
  3 (those two plus `test_metrics_initial_state`), and the port fixes the third.
  Tests are not in CI. Follow-up needed.
- The upstream test port needed SeaORM 2 raw APIs (`query_all_raw`, `execute_raw`).

## Ported (branch `port/upstream-fixes`)

| Upstream | Change | Notes |
|---|---|---|
| `dc40a85a4` | Jobs no longer linger in the client after finishing | gpui plan doc dropped |
| `9aa61f14a` | Delete `watcher_old` reference code | clean |
| `eefb55e8f` | macOS shallow watch actually delivers | clean |
| `1766bd175` | Remote job activity end to end | `sd_core` renamed |
| `0b7585cf6` | Library watcher no longer opens a library still being created | Btrfs and plus-code parts already fixed in WingDrive |
| `46a54caf3` | Full blake3 transfer verification, push truncation, sync backfill pause | watcher move-out and slug collision already fixed (WATCH-003/005) |
| `d346c6344` | Periodic reconnect dials Paired devices too | clean |
| `09d7ea2ca` | Reconnect paired peers after an in-process restart | docs merged into WingDrive text |
| `aa589a767` | Presence stops flapping; Tailscale connection method | adapted to iroh 0.98 `remote_info` |
| `c1a6bbfa2` | Job system: unresumable jobs marked failed, dispatch dedup | source-store jobs not ported |
| `ebee7ae39` | Stale integration-test assertions, hermetic env-bound tests | two files kept WingDrive version |
| `4aaec8b5c` | Discovery never indexes the daemon's own data dir | batch salvage targets source store |

## Already fixed in WingDrive

Btrfs UUID parser, plus-code decoder, Linux watcher move-out, device slug
collision, ephemeral search case-insensitivity, search scope debounce/sort/
pagination (TAURI-014), watcher data-dir exclusion.

## Not applicable (upstream architecture)

Source stores, arenas, volume index map, `sources.*` operations, location
removal, records/content identity spine, preflight/journal/undo framework,
folder compare, dedupe, Photos app, GPUI client, thumbnail service rewrite
(`service/thumbs`), WebP sidecar replication, replica fetch, library join
rework (`f9bdf724b`), nightly CLI channel, NAS/titan ops notes, docs moves.

## Candidates to re-implement later (ideas, not code)

- File-op journal with undo and trash that records origin (`bcc7337b0`)
- Preflight for copy/move/delete and folder merge (`3cd44f2da`)
- Full-byte duplicate verification before deleting a copy (`ac9f266ed`)
- Bound thumbnail decode workers by free memory (`4d18db0ab`) and scale video
  frames before selection (`bf1ca21a1`) in WingDrive's thumbnail job
- statx birth time on musl builds (`81070270d`, `63ba30edd`)
- Library join fixes: inverted backfill, reverse registration (`f9bdf724b`), for M3
- Nightly channel with sha-aware self-update (`5e285d3c9`)
