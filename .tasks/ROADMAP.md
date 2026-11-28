# WingDrive Roadmap

Updated on 2026-11-28. Method and rules in [PLANNING.md](PLANNING.md).

Focus: WingDrive as a reliable **desktop file manager** (Tauri UI + CLI) before integrations
between devices.

## Sprint calendar

Since 2026-11-13 every open task has a sprint, and every task has a milestone and an assignee
(`bernardopg`). Future sprints are a backlog plan: when each one opens, review the list against what
is left.

| Sprint | Dates | Milestone | Focus |
|---|---|---|---|
| [S05](sprints/S05.md) | 2026-11-12 to 2026-11-25 | M2 | Close M2 (DESK-051, BRAND-003, CORE-021) and release 3.4.0 |
| [S06](sprints/S06.md) | 2026-11-26 to 2026-12-09 | M3, M5 | Signed updater, Explorer smoke on every PR, Windows by hand, Google for any user |
| [S07](sprints/S07.md) | 2026-12-10 to 2026-12-23 | M4 | File sync and sharing |
| [S08](sprints/S08.md) | 2026-12-24 to 2027-01-06 | M4 | Close M4 |
| [S09](sprints/S09.md) to [S14](sprints/S14.md) | 2027-01-07 to 2027-03-31 | M5 | Optional work by priority |

<!-- board:begin progress -->
## Progress

_Generated from the task front matter by the board (`just board-sync`). Do not edit by hand._

| Milestone | Phase | Done | In Progress | To Do | Progress |
|---|---|---|---|---|---|
| M0 — Base inherited from upstream | Done | 78 | 0 | 0 | 100% |
| M1 — Linux desktop file manager 1.0 | Done | 22 | 0 | 0 | 100% |
| M2 — Default file manager on Linux | Done | 75 | 1 | 0 | 99% |
| M3 — Cross-platform desktop and release | Now | 12 | 19 | 3 | 35% |
| M4 — Device integration | Later | 4 | 4 | 16 | 17% |
| M5 — Optional | Later | 32 | 26 | 86 | 22% |

| Sprint | Dates | Done | In Progress | Progress |
|---|---|---|---|---|
| [S01](sprints/S01.md) | 2026-10-01 to 2026-10-14 | 17/17 | 0 | 100% |
| [S02](sprints/S02.md) | 2026-10-15 to 2026-10-28 | 3/3 | 0 | 100% |
| [S03](sprints/S03.md) | 2026-10-07 to 2026-10-28 | 32/32 | 0 | 100% |
| [S04](sprints/S04.md) | 2026-10-29 to 2026-11-11 | 37/37 | 0 | 100% |
| [S05](sprints/S05.md) | 2026-11-12 to 2026-11-25 | 27/32 | 2 | 84% |
| **[S06](sprints/S06.md)** (current) | 2026-11-26 to 2026-12-09 | 11/38 | 23 | 29% |
| [S07](sprints/S07.md) | 2026-12-10 to 2026-12-23 | 6/20 | 7 | 30% |
| [S08](sprints/S08.md) | 2026-12-24 to 2027-01-06 | 1/22 | 1 | 5% |
| [S09](sprints/S09.md) | 2027-01-07 to 2027-01-20 | 0/25 | 12 | 0% |
| [S10](sprints/S10.md) | 2027-01-21 to 2027-02-03 | 0/15 | 5 | 0% |
| [S11](sprints/S11.md) | 2027-02-04 to 2027-02-17 | 0/14 | 0 | 0% |
| [S12](sprints/S12.md) | 2027-02-18 to 2027-03-03 | 0/14 | 0 | 0% |
| [S13](sprints/S13.md) | 2027-03-04 to 2027-03-17 | 2/12 | 0 | 17% |
| [S14](sprints/S14.md) | 2027-03-18 to 2027-03-31 | 0/10 | 0 | 0% |
<!-- board:end progress -->

## Done — M0: Base inherited from upstream

Tasks finished in Spacedrive before the fork, with no milestone until 2026-11-13. They stay as a
record of what WingDrive inherited ready; none is revalidated here (M4 revalidates `NET-000` and
`LSYNC-000`). List: `cargo run --bin task-validator -- list --milestone M0`.

## Done — M1: Linux desktop file manager 1.0

Result: on Linux, browse, open, copy, move, rename, create folders and delete files from the UI and
the CLI, with no known bug, running from a production bundle.

Epics: `TAURI-000`, `EXPL-000`, `CLI-000` (extended by `CLI-001`).

Finished in S01: WATCH-003, WATCH-004, CLI-001, EXPL-001/002/003/005/006/007, TAURI-008/009/010/011,
DEV-003, BRAND-001 and the rename to `wing`. Release `v2.0.0-alpha.6` published on GitHub and on the
AUR (`wingdrive-bin`).

Rest of M1, in order:

| Order | Task | Title | Sprint |
|---|---|---|---|
| 1 | TAURI-013 | Linux stabilization: data safety (Done) | S01 |
| 2 | — | The next tag updates the AUR with no manual step (Done) | S01 |
| 3 | TAURI-014 | Runtime robustness and search parity (Done) | S02 |
| 4 | TAURI-006 | Close the Linux regression matrix (Done) | S02 |
| 5 | BRAND-002 | Wingbot, WingUI, Wingdrop and Wings (Done) | S02 |

Out of order, already Done: the Apache-2.0 license and the port of upstream fixes (#114, #115).
Release `v2.0.0-alpha.7` ships TAURI-013, TAURI-014 and those fixes; see the
[CHANGELOG](../CHANGELOG.md).

M1 closes when these five are Done and a release ships with no known bug.

## Done — M2: Default file manager on Linux

Top priority by the user's decision (2026-10-07). Result: WingDrive replaces Dolphin, Nautilus or
Yazi in daily use. `xdg-open` on folders and "Show in folder" from other apps open WingDrive, it
stays resident in the tray and opens at once, and it has the basic operations of a file manager.
Navigation stays graphical; a Yazi-style modal mode is out of scope.

Epic: `DESK-000`. Sprints S03 to S05.

| Order | Task | Title |
|---|---|---|
| 1 | DESK-001 | XDG integration, command-line paths and single instance |
| 2 | DESK-003 | Background in the tray and start with the session |
| 3 | DESK-002 | D-Bus service `org.freedesktop.FileManager1` |
| 4 | DESK-004 | Real "Open with" on Linux |
| 5 | DESK-005 | "Open terminal here" |
| 6 | DESK-006 | New empty file and symbolic link |
| 7 | DESK-007 | Permissions and owner in properties |
| 8 | DESK-008 | Compress and extract |
| 9 | DESK-009 | Network and devices through GIO/gvfs (MTP, SMB, SFTP) |
| 10 | DESK-010 | Startup latency and memory budget |

Prerequisite: TAURI-015 (AUR package with the system GTK/WebKit, daemon stopped on quit).

State on 2026-11-02: S01, S02 and S03 closed. Done in M2: DESK-001 to DESK-012, DESK-014,
DESK-015, DESK-018 to DESK-032, DESK-034 to DESK-040, TAURI-015, INDEX-013, INDEX-014,
NET-005 to NET-007, SEC-010, WATCH-007, WATCH-008 and the upstream ports (#230, #234, #235). The
check on the 2026-11-02 package closed 16 tasks at once (log of [S04](sprints/S04.md)). What was
left to close M2:

| Task | What was left | Sprint |
|---|---|---|
| BRAND-003 | Done on 2026-11-15: Wings sidebar on the desktop and sync with the moto g(7) on 3.3.1 | S05 |
| DESK-051 | Done on 2026-11-15: cold login with `wingdrive-bin` 3.3.1 checked by the author | S05 |
| CORE-021 | Done on 2026-11-15: 15 Explorer smoke rounds with no empty folder | S05 |
| DESK-097 | Done on 2026-11-15: a login with `--hidden` passed to the open instance does not show the window; "Start minimized to the tray" setting | S05 |
| DESK-000 | Done on 2026-11-15 | S05 |

DESK-043 and DESK-090 closed on 2026-11-14: ten `v3.3.0` sessions with 15 tabs stayed under
550 MiB, and the renderer's high mode did not come back even with `v3.0.0`.

INDEX-010, DESK-041 and DESK-042 closed on 2026-11-03. DESK-017 (several days of use of the AUR
package, reported by the author) and DESK-033 (real `pkexec` and polkit on the author's machine)
closed on 2026-11-05, together with WATCH-006 (connection at login confirmed in the journal),
DESK-016 (dual pane in the package) and DESK-013 (full pt-BR review on Crowdin and AppImage checked
on Hyprland).

**M2 closed on 2026-11-15.** The author checked DESK-051 with `wingdrive-bin` `3.3.1` (cold login
and upload in Chrome), BRAND-003 finished with the Wings sidebar on the desktop and sync with a phone
on `3.3.1`, CORE-021 passed 15 Explorer smoke rounds, and DESK-097 (window shown at login) was
fixed. With every child Done and the day of use from DESK-017, DESK-000 closed. The coverage tasks
that were in M2 moved to M3, where their targets are.

[COV-07](testing/COV-07-desktop-native.md) was reopened in M2 on 2026-11-19 for the target of 100%
line coverage of the desktop shell. The default file manager delivery closed on 2026-11-15, and M3
stays the focus.

## Now — M3: Cross-platform desktop and release

Result: the same level as M1 on macOS and Windows, with a signed public release.

State on 2026-11-05: `v3.0.0` shipped the first Windows installers (`.exe` and `.msi`, unsigned)
next to the Linux packages. The Windows CI jobs are required checks on `main`. Windows is in beta:
everything passes in CI and in the installer smoke, but nobody has tested it by hand on a real
machine. macOS still only builds from source.

Done in M3: `TAURI-016` (Windows base, CI and installers) and `FILE-007` (preflight for copy, move
and delete).

State on 2026-11-10: `v3.1.0` brings FILE-008 undo for copy, overwrite, move and trash to the core,
the CLI and the web UI, and copies that lose no data when canceled or failed.

| Order | Task | What is left | Sprint |
|---|---|---|---|
| 1 | REL-001 | Authenticode credentials and validation, Apple signing and notarization, Google Play key | S06 |
| 2 | TAURI-018 | Explorer smoke (`scripts/release/explorer.cjs`) on every pull request and 10 scheduled runs; job in `ci.yml` since 2026-11-15 | S05 |
| 3 | CORE-022 | Daemon and app logs free of noise from normal conditions (`/tmp` without permission, temporary files, teardown) | S06 |
| 4 | TAURI-019 | Daemon on Windows without a console window, with a tray icon (needs a Windows machine) | S07 |
| 5 | TAURI-012 | Regression matrix by hand: Windows round with the checks left from TAURI-016; macOS once there is a Mac | S06 |
| 6 | CORE-014 | "Open with" and opening files on Windows and macOS, checked in the TAURI-012 round | S06 |
| 7 | TAURI-017 | Retry as administrator on Windows through UAC; waits for the REL-001 certificate | — |
| 8 | CLI-002 | `wing update` finds no binaries in WingDrive releases (Done on 2026-11-05) | — |
| 9 | FILE-008 | Done on 2026-11-12: every Explorer undo goes through the core journal (#378) | — |
| 10 | TAURI-000 | Closes when REL-001, TAURI-018 and TAURI-012 close | S06 |

The AppImage and the Windows installers have signed updates since `v3.4.0`. Since `v3.5.0`,
Authenticode signing of the Windows installers turns on when the credentials are set. That keeps
REL-001 and the per-platform validation in progress.

M3 closes with TAURI-012 Done on Windows and macOS, and a signed release on every platform that has
a credential.

### Test coverage in M2 and M3

At the user's request on 2026-11-03, epic `COV-00` widens the coverage of the core, desktop and CLI
with the DEV-006 infrastructure. [Plan and targets](testing/README.md). Tasks `COV-01` to `COV-12`
have dependencies and verifiable criteria. On 2026-11-10, COV-01 to COV-04, COV-06 and COV-07 are
In Progress, with suites on `main` (#343, #344, #358, #359) and without the LCOV measurement that
closes each target; the rest stay in the backlog, without changing the S04/S05 commitment until
planning has capacity.

Contracts and critical regressions go into M2; M3 closes integration and gates with targets of 50%
for the core, 75% for the desktop, 60% for the CLI and 60% for the React interface, plus 80% of
changed production lines. They are future targets, checked by the full report, with no code excluded
to raise the average. The epic closes with every child Done and evidence on main.

## Later — M4: Device integration

Reliable pairing, Spacedrop, library sync and file sync between desktops.

- `FSYNC-000` File Sync
- `SHARE-001` sharing architecture
- Revalidate `NET-000` and `LSYNC-000` (marked Done upstream) in real use

## Later — M5: Optional

Cloud (`CLOUD-000`, `CLOUD-003`), extensions (`PLUG-000`), Spacebot (`TAURI-004`), AI (`AI-000`),
mobile (`RES-000`, `SEC-009`). They come in only after M4, unless explicitly decided.

Exception decided on 2026-11-12: third-party cloud integrations (epic `INTG-000`). The Google
integration (`CLOUD-004`: Drive as a volume, Google Photos, folder sync with the cloud) entered S05
at the author's request and closes the copy part of `FILE-003`. For any user to connect by signing
in alone, INTG-001 to INTG-005 are left (built-in OAuth client, privacy policy, Google verification,
real validation and reconnection), planned in [S06](sprints/S06.md). Then come the provider
framework (INTG-020) and Dropbox, OneDrive, Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA,
Proton Drive, object storage, SFTP and SMB (INTG-021 to INTG-031).

On 2026-11-13 signing in with the button alone worked with real Google: each provider has two
modes, "Account" (WingDrive's client built into the release, as GNOME Online Accounts does) and an
own client (Google Cloud or Microsoft Entra), chosen from a catalog of cards in Settings >
Integrations (INTG-010). OneDrive (INTG-022) already signs in through the browser; the WingDrive app
still has to be registered with Microsoft. INTG-001 to INTG-004 moved forward in S05.

Exception decided on 2026-11-24: an adapter store (epic `ADAPT-000`, folder `.tasks/adapters/`).
Sources > Add source shows "No adapters installed" on every install, because no package ships the
adapters. Instead of bundling them, each adapter gets its own repository, a signed registry lists
them, bots review submissions, and the app installs adapters from a store with cards, as
DankMaterialShell does with plugins. The store platform (ADAPT-001, 003, 004, 005, 007, 008, 009)
enters S06; the Obsidian, GitHub, Thunderbird and coding agent session adapters and the
wingdrive.org/adapters catalog follow in S07. The other adapters stay in the backlog.

Exception decided on 2026-10-17: the mobile epic `MOB-000` (working Android, fast dev loop, parity
with the desktop) was pulled forward. MOB-001 to MOB-005 enter S03.

## Backlog inherited from upstream

Tasks created upstream that were still open; since 2026-11-13 all have `bernardopg` as assignee. In
the 2026-10-16 audit they got the `upstream-backlog` tag and a "WingDrive triage" note in the file,
with nothing deleted:

- M4 (20 tasks): file sync (`FSYNC-*`), public sharing (`SHARE-*`), Spacedrop and Tailscale
  (`NET-003`, `NET-004`), sidecar sync between devices (`VSS-004`) and `FILE-006`.
- M5 (45 tasks): AI, cloud, extensions, semantic search (`SEARCH-*`), encryption and RBAC (`SEC-*`),
  mobile and virtual volumes, sidecars, collections and ingestion, inherited interface epics
  (`UI-000`, `UI-001`, `MEDIA-000`, `NAV-000`, `SETS-000`, `SRCH-000`).
- Closed as obsolete: `CORE-012` (Swift type registry, no Swift app in the repository) and
  `CORE-013` (replaced by `useNormalizedQuery` in `ts-client`).

List: `cargo run --bin task-validator -- list --milestone M5 --status "To Do"`.

## History

| Date | Change |
|---|---|
| 2026-11-28 | Planning text in English: board tables, the roadmap and the open sprints S06 to S14 translated; closed sprints S03 to S05 keep their logs as written. |
| 2026-11-20 | `v3.6.0` preparation: local operations, favorites, tags and trash in the Android Explorer (MOB-007, #431), fixes in the desktop, CLI and core. COV-07 stays open. |
| 2026-11-17 | Release `v3.5.0`: Google Picker for `drive.file` accounts and one fewer Photos scope (INTG-011, INTG-003, #428), reply to Google's OAuth verification; Windows (#420, #421) and Android (#416 to #419) fixes, logs from the tray (#423), conditional Authenticode (#422). |
| 2026-11-17 | Release `v3.4.1`: Rust 1.100, Dependabot and untracked dependencies in a single PR, code scanning alerts resolved (#410), film and media kit on the site (#412), Rust CI with one feature set (#413) |
| 2026-11-16 | Release `v3.4.0`: signed updates for the AppImage and Windows (REL-001), daemon without a console on Windows (TAURI-019), `drive.file` in WingDrive's Google client (INTG-003, #407; Google verification submitted, Data Access under review), task board and MCP (DEV-007, #406), CORE-022, CORE-023, MOB-017 and Wings (BRAND-003) |
| 2026-11-15 | **M2 Done**: DESK-051 checked by the author, BRAND-003 with sidebar and phone sync on 3.3.1, CORE-021 with 15 smoke rounds, DESK-097 (window at login) fixed; DESK-000 Done. M3 becomes Now. New tasks: TAURI-018 (Explorer smoke in CI), TAURI-019 (Windows daemon in the background), CORE-022 (log noise), CORE-023 (full sync in a loop), MOB-017 (Android review). Coverage on `main`: core 53.7%, desktop 90.7%, CLI 65.9%, interface 74.5%; M3 targets met and the open COV tasks moved to M3 |
| 2026-11-15 | Release `v3.3.1` and Wings in the code (#397). Desktop app plan: S05 closes M2 with BRAND-003, DESK-051 and CORE-021 (from S06) and ships 3.4.0; S06 opens with the signed updater (REL-001), the new TAURI-018 (Explorer smoke on every pull request) and the TAURI-012 Windows round before the integrations. The Android build works with `CFLAGS=-march=native` on the host |
| 2026-11-13 | Release `v3.3.0` (#393): Google sign-in with one button through WingDrive's OAuth client built into the release (repository secrets), own-client mode, OneDrive sign-in through the browser, integration catalog, `/privacy` and `/service-terms` |
| 2026-11-13 | Integrations: "Account" and own-client modes with a card catalog (INTG-010), OneDrive sign-in through the browser (INTG-022), WingDrive's Google project and OAuth client with real sign-in working (INTG-001, INTG-004), `/privacy` and `/service-terms` on the site (INTG-002) |
| 2026-11-13 | Release `v3.2.0` (#383 to #389): Google integration with Drive and Photos in several accounts (CLOUD-004), operations and editing on cloud volumes, cloud sync, COV-07 Done with 91.6% coverage of the desktop shell and the bugs found fixed, `sdPath` to `wdPath` (BRAND-003) |
| 2026-11-13 | Every task assigned to `bernardopg`, with a milestone (M0 for those finished upstream) and a sprint; sprints S07 to S14 planned |
| 2026-11-12 | Release `v3.1.1` (#368 to #381): `FileChooser` portal (DESK-051), undo of rename and new items in Tauri (FILE-008), fixes DESK-090 to DESK-096, CORE-020, CLI-010, about 175 tests and Dependabot updates |
| 2026-11-12 | PRs #373 to #379: site with automatic deploy and Discord in English; FILE-008 Done; DESK-051 with the `FileChooser` portal backend (browsers and the package still to check); the S03/S04 revalidation found DESK-090 to DESK-096; coverage with CORE-020, CLI-010 and DESK-060 |
| 2026-11-10 | Release `v3.1.0` in preparation (#347 to #360): FILE-008 with undo in the core, CLI and web UI (the Tauri side is left); COV-03 with no data loss on a canceled copy; MOB-013 and MOB-014 Done; bundled SQLite updated (RUSTSEC-2026-0459); COV-04, COV-06 and COV-07 In Progress with 79 tests and seven bugs fixed |
| 2026-11-05 | M2: WATCH-006, DESK-016 and DESK-013 Done; review of the 2160 pt-BR strings (46 fixes); ten DESK-043 rounds with `v3.0.0` (stable, 4 of 10 under 550 MiB). DESK-043 is left to close DESK-000 |
| 2026-11-05 | Release `v3.0.0` with the first Windows installers (#331 to #334). TAURI-016 Done, Windows jobs become required checks (#336) and the README marks Windows as beta (#335). TAURI-016 leftovers spread out: checks by hand in TAURI-012, signing in REL-001, UAC in the new TAURI-017. New CLI-002 (`wing update` with no binaries in the release). M3 table with order |
| 2026-11-03 | COV-00 and COV-01 to COV-12: progressive coverage targets, core/desktop/CLI first and evidence criteria, without widening sprints already committed |
| 2026-11-02 | WingDrive independent (#310): FORK-* tasks removed from the tracker (history in git); NET-007 inherits the open criterion of FORK-007, REL-001 (S05) the signing from FORK-002, FORK-009 dropped; upstream surveys in `docs/design/history/`. S01 closed with review and retro. The package check closes 16 S04 tasks and finds DESK-039 to DESK-042; release 2.4.0 |
| 2026-10-28 | #296 merged; S02 and S03 closed with review and retro; S03 leftovers (DESK-021, DESK-023, DESK-024, WATCH-006, MOB-005, MOB-009) in S04; DESK-034 and the two DESK-029 failures (Git badge on children and status bar count) enter S04; DESK-016 gets design notes |
| 2026-10-28 | S04: DESK-029 to DESK-033 and DESK-035 with code ready (PRs #296 and #298), In Progress until the check on the Linux build; quick view of PDF, text and code (#295); pdfjs-dist 6.4 closes Dependabot alert 47 (#297); DESK-034 postponed |
| 2026-10-17 | Review of the mobile app on the Moto G7 and the emulator: epic MOB-000 with 11 tasks in `.tasks/mobile/`; MOB-001 to MOB-005 pulled into S03 |
| 2026-10-16 | `.tasks/` audit: S03 board updated, S04 and S05 planned; 65 tasks inherited from upstream parked in M4/M5; EXPL-000 and SEC-008 Done; CORE-012 and CORE-013 obsolete |
| 2026-10-15 | M2 almost closed; candidates DESK-015, DESK-016 and DESK-017 opened; mobile app translated and with typecheck in CI |
| 2026-10-07 | New M2 "Default file manager on Linux" (DESK-000) as top priority; the old M2–M4 become M3–M5 |
| 2026-10-06 | M1 complete: TAURI-006 and BRAND-002 Done; INDEX-010 integrated; multi-device sync fixed |
| 2026-10-06 | FORK-004, TAURI-014 and FORK-005 Done; Apache-2.0 license; release `v2.0.0-alpha.7` |
| 2026-10-05 | Now updated: alpha.6 published on the AUR; TAURI-013 split (TAURI-014); BRAND-002 and TAURI-006 in S02 |
| 2026-10-01 | Roadmap created; M1 defined from the review `docs/design/history/TAURI_FILEMANAGER_REVIEW_2026-09-03.md` |
