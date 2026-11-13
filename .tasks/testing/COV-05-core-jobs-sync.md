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
