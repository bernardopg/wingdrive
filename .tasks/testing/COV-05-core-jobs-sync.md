---
id: COV-05
title: "Cobrir jobs, checkpoints e sincronização entre dispositivos"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, core]
last_updated: 2026-11-22
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

## Offline pairing sessions 2026-11-19

An isolated handler fixture creates local session codes, rejects duplicate joins, removes expired
sessions, and checks the selected code through cancellation. The old lookup took the last value
from a hash map. The direct algorithm reproducer found an older code after the third insertion.
See [BUGS.md](BUGS.md) and `coverage/goal/pairing/hash-map-before.log`.

The three instrumented cases passed twice, including the signed exchange between two
temporary registries, persistence on both peers and matching directional keys. The focused
pairing profile exercised 230 lines of the handler, 165 lines on the initiator and 126 on
the joiner. Logs and LCOV are under `coverage/goal/next/pairing/` in the primary checkout.

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

- [x] Testar retomada de checkpoint, cancelamento, retries, estados terminais e limpeza.
- [x] Cobrir conflitos, desconexão, backfill e idempotência com dois dispositivos temporários.
- [x] Reaproveitar sync_metrics_test, sync_realtime_test e sync_backfill_test.

## 2026-11-22 targeted job/sync coverage

Added isolated tests following existing fixtures/patterns (no external services):

- `core/src/service/sync/retry_queue.rs` unit tests: due-entry exponential backoff, max-retry drop, and due-only selection across devices (4 tests).
- `core/tests/job_lifecycle_test.rs`: cancel a running indexing job persists `Cancelled`; `cleanup_history` removes only old terminal `job_history` rows; terminal completed job rejects resume and reports info (3 tests).
- `core/tests/sync_resilience_test.rs` (TwoDeviceHarnessBuilder, like sync_realtime/sync_metrics): disconnect blocks convergence then fresh post-reconnect changes converge; replaying the same shared change is idempotent with no duplicate rows (2 tests).

Verification attempted in `/home/bitter/.cache/opencode/wt/cov-05` on 2026-11-22:

- `cargo fmt --all` — applied formatting fixes to new tests.
- `CARGO_BUILD_JOBS=2 cargo test -p wing-core retry_queue --lib -- --nocapture` — **not completed**; rustc for `wing-core` lib tests was SIGTERM'd (signal 15) during compilation.
- `CARGO_BUILD_JOBS=1 cargo test -p wing-core --test job_lifecycle_test --no-run` — **not completed**; rustc for `wing-core` was SIGTERM'd (signal 15) during compilation.

Environment note: concurrent rustc/cc1 processes from other worktrees were present on the host (`ps` showed large rustc jobs in cov-04/wt-bitter-ws-cov). I did not kill unrelated processes. Results are unverified locally due memory pressure; CI or a quiet host should run: `cargo test -p wing-core retry_queue --lib` and `cargo test -p wing-core --test job_lifecycle_test --test sync_resilience_test -- --nocapture`.

## Acceptance Criteria

- [ ] Ao menos 70% nos módulos selecionados de job/sync com contagens por arquivo.
- [ ] Sem operações duplicadas após retomar ou reenviar eventos.
- [ ] Core atinge pelo menos 65% de linhas no relatório completo, sem reduzir denominador para cumprir a meta.

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

## Manual resume from persisted state

A paused copy row with serialized job state is inserted into a disposable job database.
The test resumes it by ID through JobManager, verifies the copied bytes and checks the
persisted terminal status. It also checks missing-job errors and the refusal to resume
a terminal job. All three instrumented shutdown and resume tests passed on Linux.
See `coverage/goal/next/job-resume-tests.log` in the primary checkout.

## File transfer stream contracts

The instrumented file transfer suite passed 14 tests on Linux, including the new push and
pull cases. The tests run the framed stream handler in memory with temporary files. Push
checks the destination bytes, terminal session and acknowledgment. Pull checks metadata,
chunk order, per-chunk hashes and the final checksum. The log is
`coverage/goal/next/transfer-stream-tests.log` in the primary checkout.

### GitHub Actions · remoção de localização durante indexação

Os PRs #468 e #470 expuseram a corrida de cancelamento em `location_remove_jobs_test`.
O teste exige estado terminal `Cancelled` antes de apagar a localização.
Veja [registro de bugs](BUGS.md#core-remover-uma-localização-retornava-antes-de-o-indexador-parar).
