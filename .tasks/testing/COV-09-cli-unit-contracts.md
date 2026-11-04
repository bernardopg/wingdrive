---
id: COV-09
title: "Cobrir parsing, configuração e saída do CLI"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, cli]
last_updated: 2026-11-03
---

## Description

O CLI parte de 0,4%; parsing e persistência precisam de testes que cubram comandos e erros sem depender de serviços pessoais.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Expandir a suíte inicial para todos os domínios Clap, aliases, flags e valores inválidos.
- [ ] Cobrir seleção de biblioteca, migração de configuração, erros e saída human/JSON.
- [ ] Testar resolução de caminhos e conversão de args para os inputs reais do core.

## Acceptance Criteria

- [ ] CLI atinge pelo menos 25% na primeira etapa; parsing/configuração selecionados atingem 85%.
- [ ] JSON é validado estruturalmente e erros preservam contexto útil.
- [ ] Fixtures temporárias e todos os testes de wing-cli executados pelo gate local e CI.
