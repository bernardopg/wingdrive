---
id: COV-05
title: "Cobrir jobs, checkpoints e sincronização entre dispositivos"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, core]
last_updated: 2026-11-03
---

## Description

Retomada e sincronização precisam de evidência sobre recuperação de falhas e consistência, além do caminho feliz.

## Offline backfill evidence 2026-11-19

The Linux gate now runs `file_sync_simple_test` and `sync_backfill_race_test`. The backfill
fixture uses an in-memory transport, disposable core directories and 128 generated files. It
checks the exact file names on both peers after sequential and concurrent indexing and rejects
duplicates. A third case exercises a colliding device slug. The fixture propagates indexing and
backfill errors, bounds the convergence wait and shuts down both cores.

The final instrumented run and its repeat passed all 10 tests with zero failures, ignored tests or
compiler warnings. Merging the two suite profiles into the partial baseline for the unchanged
core source increased covered lines from 47,720/91,497 to 48,165/91,497. For
`core/src/service/sync/`, the partial comparison is 2,462/4,876 to 2,751/4,876. This is an
incremental comparison and the full published component measurement remains separate.
Logs and LCOV are in `coverage/goal/core-sync-backfill-final.log` and
`coverage/goal/core-sync-backfill/`.

## Dependencies

COV-04. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Testar retomada de checkpoint, cancelamento, retries, estados terminais e limpeza.
- [ ] Cobrir conflitos, desconexão, backfill e idempotência com dois dispositivos temporários.
- [ ] Reaproveitar sync_metrics_test, sync_realtime_test e sync_backfill_test.

## Acceptance Criteria

- [ ] Ao menos 70% nos módulos selecionados de job/sync com contagens por arquivo.
- [ ] Sem operações duplicadas após retomar ou reenviar eventos.
- [ ] Core atinge pelo menos 50% de linhas no relatório completo, sem reduzir denominador para cumprir a meta.
