---
id: COV-01
title: "Medir baseline e inventariar lacunas por componente"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, ci]
last_updated: 2026-11-03
---

## Description

A medição existente publica números, mas não define metas nem prioriza os arquivos de maior risco.

## Dependencies

DEV-006. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Fixar SHA, comando, ambiente e contagens cobertas/instrumentadas do relatório de referência.
- [ ] Inventariar arquivos e testes ausentes de cada runner, incluindo os não instrumentados.
- [ ] Registrar os 20 arquivos prioritários de core, desktop e CLI e repetir a medição no branch.

## Acceptance Criteria

- [ ] Baseline reproduzível com LCOV e summary.json vinculados ao mesmo SHA.
- [ ] Inventário diferencia código não executado, não compilado e não importado.
- [ ] Nenhuma melhora declarada apenas pela quantidade de testes.

## Evidence

Referência publicada verificada em 2026-11-03: SHA `1064b6b8b6e94f5b7040e3c112e784d5c0e57271`,
relatório gerado às 17:58 UTC pelo workflow Coverage em Ubuntu 24.04 (GitHub Actions). Core: 15.875/81.068 (19,6%); desktop: 2.702/5.981 (45,2%);
CLI: 20/5.128 (0,4%); interface: 2.189/5.527 (39,6%).

Fonte: `gh api "repos/bernardopg/wingdrive/contents/summary.json?ref=badges"
-H "Accept: application/vnd.github.raw+json"`. Trata-se da medição publicada do main,
não de uma medição local nova. Inventário e reprodução completa continuam pendentes.

O artefato Rust foi baixado do run concluído `41686685198`; os 20 maiores déficits por
componente e arquivos fora desse LCOV estão em [INVENTORY.md](INVENTORY.md). A auditoria
frontend, a exclusão de testes inline e a reprodução local completa seguem abertas.
