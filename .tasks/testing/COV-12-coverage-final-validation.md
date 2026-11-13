---
id: COV-12
title: "Auditar metas, estabilidade e evidências do épico"
status: To Do
assignee: bernardopg
parent: COV-00
priority: Medium
milestone: M3
sprint: S06
tags: [coverage, testing, ci]
last_updated: 2026-11-03
---

## Description

Fechar o épico exige medição completa e reproduzível, com limites por plataforma e sem confundir teste passando com cobertura.

## Dependencies

COV-03 a COV-11. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Repetir suites completas três vezes para achar intermitência e vazamento de estado.
- [ ] Comparar LCOV antes/depois por componente e arquivos prioritários e registrar duração.
- [ ] Checar CI terminal, publicação de badges/Pages e matriz nativa de M3.

## Acceptance Criteria

- [ ] Todas as metas do COV-00 satisfeitas no mesmo SHA do main com relatório completo.
- [ ] Falhas em qualquer componente não reutilizam números antigos como se fossem atuais.
- [ ] Filhas Done somente com critérios marcados, comandos, artefatos e merge, seguindo PLANNING.md.
