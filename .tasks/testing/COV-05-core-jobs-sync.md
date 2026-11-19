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

### Checkpoints de miniaturas

Os testes do formato MessagePack cobrem retomada sem local, variantes personalizadas e seleção
vazia explícita. Também carregam quatro registros antigos, com local e progresso ausentes,
presentes em conjunto ou presentes isoladamente. O teste de compatibilidade verifica os
parâmetros originais, o progresso e a seleção padrão aplicada quando variantes não foram gravadas.
O diagnóstico e a correção constam no [registro de bugs](BUGS.md#core-checkpoints-de-miniaturas-perdiam-opções-e-podiam-impedir-a-retomada).

O ensaio isolado compilou as estruturas de serialização contra Serde e MessagePack da workspace.
Três casos falharam no código anterior e os quatro passaram com a correção. O ensaio verifica
o formato persistido. A execução da fila de miniaturas e a geração de imagens não fazem parte
dessa medição. Logs em `coverage/goal/next/thumbnail-checkpoint-*.log`.

## Implementation Steps

- [ ] Testar retomada de checkpoint, cancelamento, retries, estados terminais e limpeza.
- [ ] Cobrir conflitos, desconexão, backfill e idempotência com dois dispositivos temporários.
- [ ] Reaproveitar sync_metrics_test, sync_realtime_test e sync_backfill_test.

## Acceptance Criteria

- [ ] Ao menos 70% nos módulos selecionados de job/sync com contagens por arquivo.
- [ ] Sem operações duplicadas após retomar ou reenviar eventos.
- [ ] Core atinge pelo menos 50% de linhas no relatório completo, sem reduzir denominador para cumprir a meta.

## Job failure diagnostics 2026-11-19

The existing `own_data_location_test` was missing from the Linux gate. Its first instrumented run
logged that the indexer correctly refused the daemon data folder but failed with an empty job
summary message. The isolated rerun disables networking explicitly and records the same failure.
The original test's network startup was incidental and is removed from the fixture. See
[BUGS.md](BUGS.md) for the cause. Logs are retained in `coverage/goal/core-offline-sync.log` and
`coverage/goal/job-failure-before.log`.

The corrected `own_data_location_test` now runs in the Linux gate and verifies the failure
message in the list, details and matching event. LLVM runs of `own_data_location_test`,
`job_shutdown_test`, `copy_failure_test`, `location_remove_jobs_test` and `event_system_test`
passed 25 tests with zero failures, ignored tests or compiler warnings.
The successful run is in `coverage/goal/job-failure-after.log`.
