---
id: COV-12
title: "Auditar metas, estabilidade e evidências do épico"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: Medium
milestone: M3
sprint: S06
tags: [coverage, testing, ci]
last_updated: 2026-11-22
---

## Description

Fechar o épico exige medição completa e reproduzível, com limites por plataforma e sem confundir teste passando com cobertura.

## Dependencies

COV-03 a COV-11. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Repetir suites completas três vezes para achar intermitência e vazamento de estado.
- [ ] Comparar LCOV antes/depois por componente e arquivos prioritários e registrar duração.
- [ ] Checar CI terminal, publicação de badges/Pages e matriz nativa de M3.

## Acceptance Criteria

- [ ] Todas as metas do COV-00 satisfeitas no mesmo SHA do main com relatório completo.
- [ ] Falhas em qualquer componente não reutilizam números antigos como se fossem atuais.
- [ ] Filhas Done somente com critérios marcados, comandos, artefatos e merge, seguindo PLANNING.md.

## Linux measurement 2026-11-19

The complete local collector measured `fa52e11186e4c1b726b8212334be2c99e4296885`, a
verification commit combining the published coverage changes and fixes through the GTK test.
The production scopes and line filters match the report script. The comparison uses the official
`main` report at `b980cc1fd3d4c97f25b7e59e8afb375ff1e38be0` as the baseline.

| Component | Baseline | Local full report | Target | Additional hits at this denominator |
|---|---:|---:|---:|---:|
| Desktop | 7,895 / 8,896 (88.7%) | 7,997 / 8,896 (89.9%) | 100% | 899 |
| Core | 47,999 / 91,497 (52.5%) | 49,122 / 91,525 (53.7%) | 65% | 10,370 |
| CLI | 4,842 / 7,288 (66.4%) | 5,360 / 7,292 (73.5%) | 75% | 109 |

Linux x86_64, rustc 1.100.0, cargo-llvm-cov 0.9.1 and Bun 1.4.4. Both collectors exited zero.
Rust logged 2,177 passed tests and four ignored occurrences. One of the four, the GTK clipboard
case, ran successfully in the collector's separate Xvfb stage. The S3 integration, 100K-entry
reconciliation timing check and privileged polkit case remain outside this run. The frontend logged
370 passed tests, including the mobile suites. There were no failed tests. Windows, macOS and real
mobile-device execution remain outside this local measurement. The focused CLI production diagnostic
excluded inline unit tests and test matrices and measured 3,471 / 5,402 (64.3%).

Diagnostics from this run:

- The Rust unit test for requests to an absent library emitted two expected sync multiplexer
  warnings. The test verifies that a request receives an error and notifications are discarded.
- The React failed-rename test emitted `permission denied` and `errors.renameIncomplete` once each
  from deliberate error fixtures. Both assertions passed.
- The shared-crate stage emitted one Cargo unused dependency warning for `wing-server` and its
  summary. The server no longer imports `tower-http`. The manifest and lockfile remove that unused
  direct dependency and its unreachable range parser.
- The first board validation for this coverage effort reported 30 pre-existing checklist warnings
  and no errors. Its full output remains in `coverage/goal/board.log`.

Raw logs, LCOV and report artifacts are in `coverage/goal/combined/`. The earlier direct LLVM
export in `partial-site/` omits the server and shared-crate stage. Use `site/summary.json` for the
complete local result. The next process, D-Bus and pairing suites are tracked in COV-05, COV-07
and COV-10. The component targets remain open.

### Full collector after the service, FileManager1, updater and pairing tests

Local verification commit `c457b9fe09872a73ffc29641acdaf74f392a0dbc` on Linux, with the
same Rust, LLVM, Bun and reporting filters. Both collectors exited zero.
The full coverage report is in `coverage/goal/next/full/site/summary.json` in the primary worktree.

| Component | Covered / instrumented | Coverage | Target | Remaining lines at this denominator |
|---|---:|---:|---:|---:|
| Desktop | 8,022 / 8,898 | 90.2% | 100% | 876 |
| Core | 49,642 / 91,525 | 54.2% | 65% | 9,850 |
| CLI | 5,781 / 7,311 | 79.1% | 75% | 0 |

Rust passed 2,188 tests across 81 reports. The four ignored occurrences have the same scope
as the previous result, with GTK exercised again on Xvfb. Frontend passed 370 tests.
The Rust collector ran without compiler warnings. The prior unused server dependency
warning was absent after the manifest and lockfile change. The frontend logged the same two
expected rename error fixtures. The CLI production-only diagnostic measured 3,892/5,421 (71.8%).
The CLI reaches its target under the official line filter on this local commit.
Desktop and core remain open, and this result awaits a full report on a merged SHA.

## Linux measurement after the GVFS and systemd fixture changes

The local verification tree at `aeeb1e1940a60272be1bec5b71b5f9853a2a8240` combines the
pairing, transfer, provider wizard, job resume, D-Bus and GVFS tests with the systemd fixture fix.
The thumbnail checkpoint repair was verified separately and is outside this report.

| Component | Covered / instrumented | Coverage | Target | Additional hits at this denominator |
|---|---:|---:|---:|---:|
| Desktop | 8,035 / 8,910 | 90.2% | 100% | 875 |
| Core | 50,120 / 91,531 | 54.8% | 65% | 9,376 |
| CLI | 5,914 / 7,312 | 80.9% | 75% | 0 |

The coverage collector stopped in its CLI systemd process stage after one fixture reported
`ETXTBSY`. The resume command used the same instrumentation and kept the profiles from the
earlier test stages. The source difference between the two verification commits affects only the
systemd test fixture. The remaining stages finished with all tests passing after the fix.
The two logs record 2,195 successful Rust cases after counting the three repeated successful
systemd cases once, and 371 frontend successes. The report includes profiles from the initial
systemd run. The CLI production-only diagnostic measured 4,025 / 5,422 (74.2%). The official
report published from `bd8031e9bd8eeea3664abfa2f90430775e688379` also measured CLI at 80.9%.
Windows and macOS remain outside the local measurement.

The Linux Rust build produced no compiler warnings. Two sync warnings came from the expected
absent-library fixture. The React error cases emitted `permission denied` and
`errors.renameIncomplete`. The GTK clipboard case ran separately under Xvfb. The S3, long
reconciliation timing and privileged polkit cases remain excluded. See COV-10 and BUGS.md for
the systemd fixture race and its 120 passing replay executions.

Local evidence is in `coverage/goal/next/with-gvfs/`, including `rust.log`,
`rust-resumed.log`, `frontend.log`, `rust.lcov` and `site/summary.json`.

## Full Linux measurement with sidecar, move and resident launch tests

Both collectors exited zero on `afae3683fb9f41401876d11b6073b28c164b0dd1`, a local
verification tree with the Linux sidecar and move suites and the desktop resident launch
test. The Rust collector passed 2,208 cases and the frontend collector passed 371.

| Component | Covered / instrumented | Coverage | Target | Additional hits at this denominator |
|---|---:|---:|---:|---:|
| Desktop | 8,051 / 8,910 | 90.4% | 100% | 859 |
| Core | 50,481 / 91,620 | 55.1% | 65% | 9,072 |
| CLI | 5,914 / 7,312 | 80.9% | 75% | 0 |

The standalone CLI production diagnostic measured 4,025 / 5,422 (74.2%). Coverage measures
the Linux shell and the configured test scope. It does not measure execution on other platforms.

The machine recorded no compiler warnings and no failed tests. The collector ran the GTK
clipboard case on private Xvfb after its initial ignore. The three excluded cases are S3,
large reconciliation timing and polkit. The only frontend errors logged were
`permission denied` and `errors.renameIncomplete` from the intentional failed-rename test.
Board validation passed with zero errors. Reopening COV-07 clears its unchecked-criterion
warning and leaves 30 warnings, including the existing S05 WIP overage.

The full logs, LCOV and JSON summary are under `coverage/goal/after-sidecars/`.

## Full Linux measurement after watcher and native desktop regressions

Both collectors exited zero on `696a845879419a74489737d09810c0921303d500`. This local
verification tree includes the persisted and ephemeral watcher suites, the concurrent MIME
regression, terminal discovery and the native default opener fallback.

| Component | Covered / instrumented | Coverage | Target | Additional hits at this denominator |
|---|---:|---:|---:|---:|
| Desktop | 8,131 / 8,960 | 90.7% | 100% | 829 |
| Core | 50,790 / 91,653 | 55.4% | 65% | 8,785 |
| CLI | 5,914 / 7,312 | 80.9% | 75% | 0 |

The Rust collector passed 2,214 cases and the frontend collector passed 371. The Rust log
contains no compiler warnings or failed tests. GTK clipboard passed in the separate Xvfb
stage. S3, the large reconciliation timing test and polkit remain excluded. The frontend
emitted the two deliberate failed-rename diagnostics, `permission denied` and
`errors.renameIncomplete`. The CLI production diagnostic measured 4,025/5,422 (74.2%).

The logs, LCOV and summary are in `coverage/goal/after-watchers/`. This component report
measures Linux execution. The real startup probe of the installed AUR binary is separate.

## Audit 2026-11-22 (COV-12)

Scope: audit only. No full coverage collectors were rerun in this pass; no 100% coverage claim is made.

### Evidence snapshot (latest recorded local Linux report)

Source: "Full Linux measurement after watcher and native desktop regressions" above, local verification tree `696a845879419a74489737d09810c0921303d500` (not a merged `main` SHA). Both collectors exited zero; Rust 2,214 passed, frontend 371 passed.

| Component | Covered / instrumented | Coverage | Target | Gap |
|---|---:|---:|---:|---:|
| Desktop (`apps/tauri/`) | 8,131 / 8,960 | 90.7% | 100% | 829 lines |
| Core (`core/`) | 50,790 / 91,653 | 55.4% | 65% | 8,785 lines at this denominator |
| CLI (`apps/cli/`) | 5,914 / 7,312 | 80.9% | 75% | target met (+0 required) |
| Interface (`packages/interface/`) | measured separately by frontend collector | COV-08 reports >=60% | 60% | needs same-SHA full report confirmation |

CLI production-only diagnostic (from same record): 4,025/5,422 (74.2%). Official component report remains the gate metric.

### COV epic status

| Task | Status | Notes |
|---|---|---|
| COV-00 | In Progress | Epic open; targets not all met on a merged main SHA. |
| COV-01 | In Progress | Baseline/inventory partially recorded; inventory completeness still open. |
| COV-02 | Done | Merged contract regression baseline. |
| COV-03 | In Progress | PR work not in open COV PR set audited here; criteria still open. |
| COV-04 | In Progress | Criteria open. |
| COV-05 | To Do | PR #502 open, has CI failures (1 FAILURE observed). |
| COV-06 | In Progress | PR #501 open, has CI failures (2 FAILURE observed). |
| COV-07 | In Progress | PR #503 merged; desktop still 90.7% vs 100% target. |
| COV-08 | In Progress | PR #498 open; checks mostly SUCCESS/pending at audit time. |
| COV-09 | In Progress | PR #504 open, mergeable, checks pending/success at audit time. |
| COV-10 | In Progress | PR #494 merged; CLI target met in local full report, COV-10 criterion text still unchecked. |
| COV-11 | To Do | PR #499 open; checks mostly SUCCESS/pending at audit time. |
| COV-12 | In Progress | This audit; final validation criteria not met. |

### Open/related PRs (observed 2026-11-22 via `gh pr view`)

| PR | Task | State | Mergeable | Checks (observed) | Note |
|---|---|---|---|---|---|
| #494 | COV-10 | MERGED | UNKNOWN | 19 SUCCESS, 2 SKIPPED | Merged. |
| #498 | COV-08 | OPEN | UNKNOWN | 17 SUCCESS, 2 SKIPPED, 2 pending/blank | No review decision. |
| #499 | COV-11 | OPEN | UNKNOWN | 17 SUCCESS, 2 SKIPPED, 2 pending/blank | No review decision. |
| #501 | COV-06 | OPEN | UNKNOWN | 20 SUCCESS, 4 SKIPPED, 2 FAILURE, 1 pending/blank | Needs CI failure triage. |
| #502 | COV-05 | OPEN | UNKNOWN | 20 SUCCESS, 4 SKIPPED, 1 FAILURE, 2 pending/blank | Needs CI failure triage. |
| #503 | COV-07 | MERGED | - | - | Merged. |
| #504 | COV-09 | OPEN | MERGEABLE | 16 SUCCESS, 2 SKIPPED, 4 pending/blank | No review decision. |

Check snapshots are point-in-time; recheck before merge.

### Gaps / blocked / unmeasured

- **Not met:** Core 55.4% < 65%; Desktop 90.7% < 100%; no same-SHA merged-`main` full report proving final COV-00 targets.
- **COV-11 gate missing:** progressive per-component ratchet not merged; altered-line >=80% gate not enforced.
- **Stability not proven:** COV-12 requires three complete suite runs to detect flakiness/state leakage; not executed.
- **Platform limits:** Linux-only evidence. Windows/macOS not measured locally. Real mobile-device execution not measured. GTK clipboard exercised under Xvfb; S3, 100K reconciliation timing, and privileged polkit remain excluded.
- **Open PR risk:** #501/#502 have observed CI failures; #498/#499/#504 have pending checks/no review.
- **Evidence hygiene:** several measurements are local verification trees, not merged main SHAs; do not reuse as final closure evidence.

### S08 / follow-up recording

No small, cheap, safe test gap was added in this audit pass (audit-only, no coverage artifacts present in worktree). Carryover for S08 planning:

- Close Core gap to 65% and Desktop gap to 100% with targeted suites guided by fresh LCOV on a merged SHA.
- Merge/fix #501, #502, #498, #499, #504; re-run full report after merges.
- Implement/merge COV-11 ratchet before claiming epic closure.
- Run three consecutive full-suite passes on same environment/SHA and record flakiness.
- Obtain/record Windows/macOS/native CI or explicit platform exclusions in COV-12 evidence.
