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
last_updated: 2026-11-19
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
