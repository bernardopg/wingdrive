---
id: COV-04
title: "Cobrir banco, indexação e reconciliação do watcher"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, core]
last_updated: 2026-11-03
---

## Description

Listagens e metadados devem permanecer consistentes quando o filesystem muda durante a indexação.

## Dependencies

COV-03. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Testar migrations em SQLite temporário, rollback e reabertura.
- [ ] Testar rename, exclusão, subtree, hidden files, symlinks e reconciliação de UUID.
- [ ] Testar eventos duplicados e concorrentes com fixtures limitadas e waits por condição.

## Acceptance Criteria

- [ ] Ao menos 70% de linhas nos módulos de indexação e persistência selecionados no inventário.
- [ ] Sem fixtures dependentes do checkout, da biblioteca do autor ou de sleeps fixos.
- [ ] Segunda execução de indexação não duplica registros nem perde tags.
