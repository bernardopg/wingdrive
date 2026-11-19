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

## Implementation Steps

- [ ] Testar retomada de checkpoint, cancelamento, retries, estados terminais e limpeza.
- [ ] Cobrir conflitos, desconexão, backfill e idempotência com dois dispositivos temporários.
- [ ] Reaproveitar sync_metrics_test, sync_realtime_test e sync_backfill_test.

## Acceptance Criteria

- [ ] Ao menos 70% nos módulos selecionados de job/sync com contagens por arquivo.
- [ ] Sem operações duplicadas após retomar ou reenviar eventos.
- [ ] Core atinge pelo menos 65% de linhas no relatório completo, sem reduzir denominador para cumprir a meta.
