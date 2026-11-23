---
id: COV-11
title: "Aplicar metas progressivas e impedir regressão de cobertura"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, ci]
last_updated: 2026-11-23
---

## Description

A cobertura precisa de um gate por componente; média global permite esconder regressões do core com testes de utilitários.

## Dependencies

COV-01, COV-05, COV-07, COV-08, COV-10. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Reusar scripts/coverage-rust.sh, coverage-frontend.sh e coverage-report.py.
- [x] Versionar pisos por componente e medir cobertura das linhas alteradas em código de produção.
- [x] Falhar para relatório ausente/incompleto, denominador inesperadamente menor ou regressão, sem atualizar baseline automaticamente.

## Acceptance Criteria

- [ ] Pisos finais: core 65%, desktop 100%, CLI 75%, interface 60%; linhas de produção alteradas pelo menos 80%.
- [x] Gate testa limites, ausência de dados e diferenças de paths com LCOV sintético.
- [ ] PR executa escopo necessário sem duplicar builds; main publica relatório completo e status por componente.

## Implementação

- `scripts/coverage-floors.json`: pisos versionados por componente, com `min_percent` progressivo, `target_percent` final e `min_lines` para detectar denominador inesperadamente menor. Baseline inicial vem do `summary.json` publicado no branch `badges` (commit `bdf6f6b7a`, 2026-11-22T06:58Z): core 56,7%/92102 linhas, desktop 91,8%/8958, CLI 80,0%/7421, interface 74,0%/7360. Os pisos atuais respeitam o estado atual; os alvos finais ficam registrados como `target_percent`.
- `scripts/coverage-ratchet.py`: gate `check` valida floors contra `summary.json`; falha para summary/floors ausente ou inválido, componente ausente/incompleto, `lines < min_lines`, regressão abaixo do piso e cobertura de linhas de produção alteradas abaixo de 80% quando `--diff-base` e LCOVs são informados. Escreve `ratchet.json` opcional. Nunca atualiza floors automaticamente.
- `scripts/coverage-report.py`: aceita `--floors` opcional e inclui `floor_percent`, `target_percent`, `min_lines` e `floor_status` por componente no `summary.json` e colunas Floor/Status no `summary.md`.
- `.github/workflows/coverage.yml`: `fetch-depth: 0`, passa `--floors` ao report, executa `coverage-ratchet.py` quando frontend e rust tiveram sucesso, inclui `ratchet.json` nos badges/site. O check de linhas alteradas usa `github.event.before` em push.

## Evidence

- 2026-11-22, Linux, branch `test/cov-11-coverage-ratchet`, worktree `/home/bitter/.cache/opencode/wt/cov-11`.
- `python3 -m py_compile scripts/coverage-ratchet.py scripts/coverage-report.py`: OK.
- `python3 -m unittest scripts/tests/test_coverage_ratchet.py`: 17 testes, OK (13,6s). Cobertura sintética de limites, ausência de dados, denominador menor, regressão, parsing/merge de LCOV com paths absolutos/relativos e filtro de testes/gerados, e diff de linhas alteradas com repo temporário.
- Smoke manual do gate com summary sintético baseado no baseline publicado: OK; regressão forçada do core 56,6% < 56,7% falhou; denominador desktop 8000 < 8958 falhou.

## Gaps

- Os pisos finais do critério (core 65%, desktop 100%, CLI 75%, interface 60%) ainda não são todos atingíveis pelo estado atual medido; o arquivo versiona alvo final e piso progressivo atual para não quebrar o gate sem evidência de cobertura.
- O gate de linhas alteradas considera linhas presentes nos registros DA do LCOV. Arquivos de produção alterados sem entrada no LCOV não são contados como instrumentados; isso evita falso positivo, mas pode subestimar lacunas quando a coleta não instrumenta o arquivo.
- O workflow de coverage continua disparando em push main/schedule/dispatch. O critério "PR executa escopo necessário sem duplicar builds" não foi totalmente implementado porque as coletas Rust/frontend instrumentadas duplicariam builds; falta desenho de escopo por paths/componentes ou reaproveitamento seguro de artefatos.
- Não executei o workflow Coverage completo localmente (custo alto, LLVM/Xvfb/Bun/mobile). A validação local cobriu scripts Python e cenários sintéticos.

## Realignment (2026-11-23)

- PR #499 merged with progressive coverage ratchet (floors/ratchet script/workflow integration). Final AC floors/PR-scope no-duplicate-builds/full workflow E2E validation remain. Remains In Progress.
