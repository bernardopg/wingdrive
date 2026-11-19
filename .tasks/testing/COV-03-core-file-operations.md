---
id: COV-03
title: "Cobrir operações de arquivos e prevenção de perda de dados"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, core]
last_updated: 2026-11-15
---

## Description

Cópia, movimentação, rename, criação e exclusão têm prioridade sobre utilitários sem risco de perda de dados.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Criar fixtures temporárias com colisões, arquivos vazios, diretórios e links simbólicos.
- [x] Exercitar overwrite, conflito, destino dentro da origem, cancelamento e falha parcial.
- [x] Verificar checksum, metadados e ausência de remoção da origem em falhas.

## Acceptance Criteria

- [ ] Ao menos 80% de linhas nos módulos alterados de core/src/ops/files, com lista explícita de arquivos.
- [x] Casos de erro e cancelamento confirmam bytes e caminhos preservados.
- [ ] LCOV antes/depois com o mesmo conjunto de arquivos e testes no gate normal.

## Evidence

- 2026-11-06: `copy_symlink_test`, `copy_conflict_test`, `delete_strategy_test` (caso novo de
  links) e `app_config_action_test` entram no gate Linux de `scripts/ci-rust.sh`. Cobrem links
  dentro de pastas copiadas e movidas, conflito surgido depois da confirmação, overwrite
  escolhido e renomeação para nome ocupado. Os bugs que acharam estão em [BUGS.md](BUGS.md).
  Ainda faltam arquivos vazios, cancelamento, falha parcial e medição por arquivo de `ops/files`.
- 2026-11-10: `core/tests/copy_failure_test.rs` (14 casos, no gate Linux de `scripts/ci-rust.sh`)
  cobre arquivos e pastas vazios, conteúdo e data de modificação, cancelamento de cópia, de
  sobrescrita e de movimentação, falha de leitura, lote com um arquivo ilegível, pasta com arquivo
  ilegível, pasta com pipe e links selecionados. O cancelamento usa um pipe nomeado alimentado por
  uma thread, para parar o job no meio de um arquivo. Sete bugs corrigidos, em [BUGS.md](BUGS.md).
  Conferido pelo CLI num daemon isolado: `wing file copy --overwrite --method streaming` de um
  arquivo de 300 KB com data de 2017 e de um link quebrado; o destino ficou com os mesmos bytes, a
  data de 2017 e o link.
  Falta a medição LCOV por arquivo de `ops/files`, que decide o critério de 80%.

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

`core/src/ops/files` is at 63.9% overall; the 80% criterion applies to the changed modules listed in the task.

### Duplicate detection with overlapping roots

The Linux integration test dispatches the duplicate job in four modes. It verifies exact
paths, duplicate counts and potential savings for overlapping folders and a path with
`..`. The instrumented baseline overcounted real files and the corrected run passed.
See [BUGS.md](BUGS.md) and `coverage/goal/after-watchers/duplicate-*.log`.

### File moves through reindexing and watchers

The Linux CI and coverage collector now run the existing move suite against temporary libraries
and directories. It verifies persistent and ephemeral moves, event delivery and identity across
reindexing and filesystem watcher paths. The four move scenarios and two fixture assertions
passed in two serial instrumented runs. Logs are in
`coverage/goal/next/file-move-{baseline,replay}.log`.
