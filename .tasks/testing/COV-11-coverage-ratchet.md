---
id: COV-11
title: "Aplicar metas progressivas e impedir regressão de cobertura"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
tags: [coverage, testing, ci]
last_updated: 2026-11-03
---

## Description

A cobertura precisa de um gate por componente; média global permite esconder regressões do core com testes de utilitários.

## Dependencies

COV-01, COV-05, COV-07, COV-08, COV-10. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Reusar scripts/coverage-rust.sh, coverage-frontend.sh e coverage-report.py.
- [ ] Versionar pisos por componente e medir cobertura das linhas alteradas em código de produção.
- [ ] Falhar para relatório ausente/incompleto, denominador inesperadamente menor ou regressão, sem atualizar baseline automaticamente.

## Acceptance Criteria

- [ ] Pisos finais: core 50%, desktop 75%, CLI 60%, interface 60%; linhas de produção alteradas pelo menos 80%.
- [ ] Gate testa limites, ausência de dados e diferenças de paths com LCOV sintético.
- [ ] PR executa escopo necessário sem duplicar builds; main publica relatório completo e status por componente.
