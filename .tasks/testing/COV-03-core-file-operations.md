---
id: COV-03
title: "Cobrir operações de arquivos e prevenção de perda de dados"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, core]
last_updated: 2026-11-03
---

## Description

Cópia, movimentação, rename, criação e exclusão têm prioridade sobre utilitários sem risco de perda de dados.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Criar fixtures temporárias com colisões, arquivos vazios, diretórios e links simbólicos.
- [ ] Exercitar overwrite, conflito, destino dentro da origem, cancelamento e falha parcial.
- [ ] Verificar checksum, metadados e ausência de remoção da origem em falhas.

## Acceptance Criteria

- [ ] Ao menos 80% de linhas nos módulos alterados de core/src/ops/files, com lista explícita de arquivos.
- [ ] Casos de erro e cancelamento confirmam bytes e caminhos preservados.
- [ ] LCOV antes/depois com o mesmo conjunto de arquivos e testes no gate normal.
