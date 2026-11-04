---
id: COV-08
title: "Cobrir fluxos do Explorer e sidebar na interface React"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, desktop]
last_updated: 2026-11-03
---

## Description

A interface compartilhada aparece em outro componente do relatório e precisa de meta própria para evitar lacunas no desktop.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Cobrir seleção, expansão, navegação, busca, filtros e eventos que invalidam queries.
- [ ] Testar copiar/recortar/colar, confirmação de exclusão, erros e restauração de foco.
- [ ] Validar labels acessíveis e catálogos en/pt; executar testes com mocks isolados por processo.

## Acceptance Criteria

- [ ] Interface packages/interface atinge pelo menos 60% das linhas instrumentadas.
- [ ] Módulos alterados alcançam 80% e testes verificam interações visíveis, não somente snapshots.
- [ ] Smoke de Explorer/sidebar usa web UI e daemon isolado; testes nativos ficam com COV-07.
