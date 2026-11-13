---
id: COV-10
title: "Testar CLI com daemon isolado e filesystem real"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, cli]
last_updated: 2026-11-03
---

## Description

Testes de parsing não provam que wing envia requests corretos, aguarda jobs e informa falhas com exit code adequado.

## Dependencies

COV-09, COV-03. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Subir daemon com diretório, instância e endpoints exclusivos de cada teste.
- [ ] Executar library/location/file/search/job/status, verificando JSON, exit code e bytes em disco.
- [ ] Cobrir daemon indisponível, timeout, biblioteca ausente e falha de operação; encerrar apenas PIDs criados pelo teste.

## Acceptance Criteria

- [ ] CLI atinge pelo menos 60% de linhas na medição completa.
- [ ] Cópia/move/rename/delete/mkdir têm cenários de sucesso e erro no CLI real.
- [ ] Sem tocar daemon, serviços, portas ou bibliotecas do autor; teardown funciona após falhas.
