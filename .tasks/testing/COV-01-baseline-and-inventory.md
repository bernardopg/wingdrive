---
id: COV-01
title: "Medir baseline e inventariar lacunas por componente"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, ci]
last_updated: 2026-11-15
---

## Description

A medição existente publica números, mas não define metas nem prioriza os arquivos de maior risco.

## Dependencies

DEV-006. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Fixar SHA, comando, ambiente e contagens cobertas/instrumentadas do relatório de referência.
- [ ] Inventariar arquivos e testes ausentes de cada runner, incluindo os não instrumentados.
- [ ] Registrar os 20 arquivos prioritários de core, desktop e CLI e repetir a medição no branch.

## Acceptance Criteria

- [x] Baseline reproduzível com LCOV e summary.json vinculados ao mesmo SHA.
- [ ] Inventário diferencia código não executado, não compilado e não importado.
- [ ] Nenhuma melhora declarada apenas pela quantidade de testes.

## Evidence

Referência publicada verificada em 2026-11-03: SHA `1064b6b8b6e94f5b7040e3c112e784d5c0e57271`,
relatório gerado às 17:58 UTC pelo workflow Coverage em Ubuntu 24.04 (GitHub Actions). Core: 15.875/81.068 (19,6%); desktop: 2.702/5.981 (45,2%);
CLI: 20/5.128 (0,4%); interface: 2.189/5.527 (39,6%).

Fonte: `gh api "repos/bernardopg/wingdrive/contents/summary.json?ref=badges"
-H "Accept: application/vnd.github.raw+json"`. Trata-se da medição publicada do main,
não de uma medição local nova. Inventário e reprodução completa continuam pendentes.

O artefato Rust foi baixado do run concluído `41686685198`; os 20 maiores déficits por
componente e arquivos fora desse LCOV estão em [INVENTORY.md](INVENTORY.md). A auditoria
frontend, a exclusão de testes inline e a reprodução local completa seguem abertas.

### Medição local no branch `test/coverage-expanded`, 2026-11-06

Coleta completa com `scripts/coverage-rust.sh`, `scripts/coverage-frontend.sh` e
`scripts/coverage-report.py`, com `HOME` e `XDG_CONFIG_HOME` temporários, em Linux. Core:
24.635/81.455 (30,2%); desktop: 3.781/6.912 (54,7%). O core passa da primeira etapa (30%); o
desktop ainda não chega aos 60%. A medição antecede a revisão do PR, que moveu testes para o
runner Linux e acrescentou casos ao desktop; o número oficial é o do workflow Coverage após o
merge. Uma medição do dia anterior (core 20,1%, desktop 49,1%) foi descartada porque a primeira
execução falhou no meio e os ignorados não entraram no relatório.

## Measurement 2026-11-15

Coverage workflow of `main` at `5c7764815` (run 43686539472), `summary.json` on the `badges`
branch and the `coverage-rust` LCOV artifact of the same run:

| Area | Lines |
|---|---|
| Core | 53.7% (48,548 / 90,473) |
| Desktop (Tauri shell and its crates) | 90.7% (7,842 / 8,647) |
| CLI | 65.9% (4,830 / 7,330) |
| Interface (React) | 74.5% (5,159 / 6,925) |
| `core/src/ops/files` | 63.9% (5,901 / 9,234) |
| `core/src/ops/indexing` | 73.7% (6,567 / 8,909) |
| `core/src/infra/db` | 54.6% (3,020 / 5,529) |
| `core/src/domain/addressing` | 76.8% (553 / 720) |
| `core/src/infra/wire` and `infra/api` | 80.3% (1,174 / 1,462) |
| `core/src/infra/job` | 43.1% (1,294 / 2,999) |
| Sync (`service/sync`, `infra/sync`) | 72.5% (5,885 / 8,121) |

The four M3 component goals (core 50%, desktop 75%, CLI 60%, interface 60%) are met on `main`.
Moved to M3 with the rest of the coverage epic, since its open criteria are module goals and gates,
not M2 work.

The baseline criterion is met: LCOV and `summary.json` come from the same SHA.
