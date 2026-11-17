---
id: CORE-024
title: Replace the unmaintained chrono crate with a maintained date and time library
status: To Do
assignee: bernardopg
priority: Medium
milestone: M5
sprint: S14
tags: [core, dependencies, security, database, sync]
related_tasks: [SEC-008, LSYNC-000]
last_updated: 2026-11-17
---

## Description

On 2026-11-17 RustSec published two notices:

- `RUSTSEC-2026-0543`: `chrono` is unmaintained, for every version. The announcement is
  chronotope/chrono#1768.
- `RUSTSEC-2026-0542`: `humantime` wraps `chrono` and is unmaintained with it.

`cargo-deny` failed every pull request, because `deny.toml` fails unmaintained notices for direct
dependencies (`unmaintained = "workspace"`). #425 adds both IDs to `[advisories] ignore`, with a
comment that says when to remove them.

**Severity:** informational, with no CVE and no CVSS score. No flaw in `chrono` is known. The risk
is forward-looking: a bug or vulnerability found from now on gets no fix release, and the
ecosystem will move away from the crate over time. Nothing has to change urgently. The ignore
entries must not become permanent, though.

### Where chrono is used (measured 2026-11-17)

**Our code:** about 1,100 uses (`chrono::`, `DateTime<Utc>`, `Utc::now`) in 226 Rust files, across
nine workspace crates:

- `wing-core`, which also enables `sea-orm/with-chrono` and `specta/chrono`;
- `wing-cli`;
- `wingdrive-sdk`;
- `wing-archive`;
- `wing-ffmpeg`;
- `wing-media-metadata`;
- `log-analyzer`, which also enables `rusqlite/chrono`;
- `task-validator`;
- `wing-bench`, which also uses `humantime-serde`.

The SeaORM entities in `core/src/infra/db/entities` hold 17 `DateTime<Utc>` fields. The RPC types
and the sync log serialize timestamps through serde. The TypeScript and Swift types are generated
from them by specta.

**Dependencies that pull chrono in** (`cargo tree -i chrono`):

| Crate | chrono required? | Alternatives it supports |
|---|---|---|
| `sea-orm` 2.0.4 | optional (`with-chrono`) | `with-time`; no `with-jiff` in 2.0.4 |
| `sea-query` 1.0.2 | optional (`with-chrono`) | `with-time`, `with-jiff` |
| `specta` 2.0.0-rc.27 | optional (`chrono`) | `time`, `jiff` |
| `rusqlite` 0.41 (`log-analyzer`) | optional (`chrono`) | turn the feature off |
| `sqlx-core` 0.9 | optional | `time` |
| `trash` 5.3 (`core`) | optional, but in its default features on Linux | `default-features = false, features = ["coinit_apartmentthreaded", "freedesktop"]` |
| `pdfium-render` 0.9.5 (`wing-images`) | **required**, no feature to turn it off | none: needs an upstream change, a fork or another PDF library |

## Alternatives

### A. Keep chrono and ignore the notices (current state after #425)

- **For:** no work, and no risk to stored data or sync.
- **Against:** no fixes from now on. Every new advisory against `chrono` has to be judged by hand.
  The ignore entries hide later notices against the same crate too.
- **Use it as:** the bridge until one of the options below lands, not the answer.

### B. A maintained chrono fork or a new maintainer

- **For:** the smallest change: a `[patch.crates-io]` entry or a rename in `Cargo.toml`. The code,
  the stored formats and the dependencies stay as they are.
- **Against:** it depends on others. Check chronotope/chrono#1768 and RustSec for a successor
  before starting C or D. A fork without a clear owner only moves the problem.

### C. Migrate to `time` (suggested)

- **For:** mature and widely used. Every dependency in the table except `pdfium-render` supports
  it today, including `sea-orm` (`with-time`), so the database layer can move now. `serde` with
  `time::serde::rfc3339` keeps the RFC 3339 text that chrono's serde writes.
- **Against:** the API is less ergonomic for time zones; `time` handles UTC offsets, not IANA
  zones. The code here works almost only in UTC, and local times are formatted in the UI, so this
  costs little.

### D. Migrate to `jiff`

- **For:** the successor the RustSec notice recommends, with the best API and time zone support.
  `sea-query` and `specta` support it already.
- **Against:** `sea-orm` 2.0.4 has no `with-jiff`, so the entities would need custom
  `ValueType` impls, or the migration waits for SeaORM. It is younger than `time`, with a smaller
  ecosystem.
- **Revisit:** if SeaORM ships `with-jiff` before this task starts, D becomes the better target.

### E. Mixed: `time` in the database layer, `jiff` elsewhere

- **Against:** two date types in one codebase means conversions at every boundary between
  entities, RPC types and sync. Rejected.

## Suggestion

Take **B** if a maintained fork with a clear owner exists when the task starts. Otherwise take
**C**, migrating to `time` in steps. Each step is one pull request that keeps every stored and
wire format byte-compatible:

1. **Formats first.** Write tests that pin today's formats:
   - the SQLite text of each `DateTime<Utc>` column, as sea-query's `with-chrono` writes it;
   - the JSON of RPC outputs and sync log entries;
   - the generated TypeScript and Swift types.

   Every later step must pass them unchanged. A phone on an older version keeps syncing with a
   newer desktop only if the formats match.
2. **Small crates:**
   - `task-validator`, `log-analyzer` (drop `rusqlite/chrono`), `wing-archive`, `wing-ffmpeg`
     and `wing-media-metadata`;
   - `wing-bench`, replacing `humantime-serde` with a small duration parser or with `jiff`'s
     `SignedDuration` parsing.
3. **`trash`:** turn off its default features in `core/Cargo.toml` and keep the platform features
   it needs.
4. **`wing-core` database layer:**
   - `sea-orm` from `with-chrono` to `with-time`, and the 17 entity fields to
     `time::OffsetDateTime`;
   - a migration test that opens a library written by the current release and reads its dates
     back.
5. **`wing-core` operations, RPC and sync,** plus `specta` from `chrono` to `time`. Regenerate the
   TypeScript and Swift types; the TypeScript type must stay `string`.
6. **`wing-cli` and `wingdrive-sdk`.** Extensions built with the SDK see a new type, so note it in
   the SDK changelog.
7. **`pdfium-render`:**
   - open an upstream issue or PR that puts `chrono` behind a feature, since it uses chrono only
     in `utils.rs` for PDF dates;
   - until then, keep it as the one crate that pulls chrono, and narrow the `deny.toml` comment to
     say so;
   - if upstream does not move, consider a `[patch.crates-io]` fork.
8. **Remove the `RUSTSEC-2026-0542` and `RUSTSEC-2026-0543` entries** from `deny.toml` when
   `cargo tree -i chrono` and `cargo tree -i humantime` come back empty.

## Acceptance Criteria

- [ ] Check chronotope/chrono#1768 and RustSec for a maintained successor; record the decision
      between B, C and D in this file
- [ ] Format tests pin the database text, the RPC and sync JSON and the generated types before
      any migration step
- [ ] No workspace crate depends on `chrono` or `humantime` directly
- [ ] A library written by the release before the migration opens, reads its dates and syncs with
      a device on the new release, both ways
- [ ] `cargo tree -i chrono` lists only `pdfium-render`, or nothing, with an upstream issue linked
      here for `pdfium-render`
- [ ] The two ignore entries are gone from `deny.toml`, or narrowed to `pdfium-render` with a reason

## Notes

- No urgency: the notices report no flaw. Revisit sooner if RustSec publishes a vulnerability
  against `chrono`, since no fix release would follow.
- `cargo-deny` is not a required check, so its failures do not block merges. A red `cargo-deny` on
  every pull request still hides real findings, which is why #425 ignores the two IDs instead of
  leaving the job red.
