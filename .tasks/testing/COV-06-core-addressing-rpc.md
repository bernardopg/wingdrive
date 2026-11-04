---
id: COV-06
title: "Cobrir endereçamento, validação e contratos RPC"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, core]
last_updated: 2026-11-03
---

## Description

Caminhos locais, remotos e de conteúdo e o registro Wire sustentam operações usadas por todos os clientes.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Cobrir WingPath e batches, URIs inválidas, Unicode e resolução sem seguir links.
- [ ] Testar registro de queries/actions e erros de método, payload e biblioteca ausente.
- [ ] Testar formatos públicos consumidos pelo ts-client sem redefinir tipos.

## Acceptance Criteria

- [ ] Ao menos 85% de linhas nos módulos selecionados de endereçamento e validação.
- [ ] Erros de protocolo e de domínio mantêm códigos e payloads esperados.
- [ ] Tipos públicos continuam atuais após generate_typescript_types.
