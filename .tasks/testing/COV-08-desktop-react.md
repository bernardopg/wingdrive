---
id: COV-08
title: "Cobrir fluxos do Explorer e sidebar na interface React"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, desktop]
last_updated: 2026-11-15
---

## Description

A interface compartilhada aparece em outro componente do relatório e precisa de meta própria para evitar lacunas no desktop.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Cobrir seleção, expansão, navegação, busca, filtros e eventos que invalidam queries.
- [ ] Testar copiar/recortar/colar, confirmação de exclusão, erros e restauração de foco.
- [ ] Validar labels acessíveis e catálogos en/pt; executar testes com mocks isolados por processo.

## Acceptance Criteria

- [x] Interface packages/interface atinge pelo menos 60% das linhas instrumentadas.
- [ ] Módulos alterados alcançam 80% e testes verificam interações visíveis, não somente snapshots.
- [ ] Smoke de Explorer/sidebar usa web UI e daemon isolado; testes nativos ficam com COV-07.

## Evidence

### 2026-11-14, branch `fix/m2-close`

Base publicada no `main` (`1e740bf87`, workflow Coverage de 2026-11-14 04:09 UTC, artefato
`coverage-frontend` do run 43548918211): interface 3.101/6.904 linhas (44,9%), 73 arquivos. Os
maiores déficits eram `routes/explorer/context.tsx` (743 linhas sem cobertura),
`components/TabManager/TabManagerContext.tsx` (659) e `routes/explorer/SelectionContext.tsx` (420).

Testes novos, cada um no seu processo como o CI roda (`scripts/coverage-frontend.sh`), com os hooks de
dados mockados por `mock.module` e as exportações reais preservadas:

- `routes/explorer/context.test.tsx`: navegação por caminho e view, voltar/avançar pelo histórico da
  aba, publicação dos arquivos da view atual, filtro por pasta, pasta operacional na view em colunas,
  ordenação com inversão de direção, modos busca/recentes/tag/filtrado, painéis e Quick Preview.
- `components/TabManager/TabManagerContext.test.tsx`: criar, alternar, reordenar, fechar e reabrir
  abas com o estado do Explorer, seleção por aba com remapeamento de UUID e troca de biblioteca,
  painel duplo, persistência e rejeição de estado salvo inválido.
- `routes/explorer/SelectionContext.test.tsx`: seleção simples, múltipla e por intervalo, foco por
  teclado, sincronização com a plataforma só na aba ativa, renomear (sucesso, nome igual, falha com
  retry de administrador) e restauração a partir da listagem.
- Hooks e utilitários: `useClipboard`, `useTheme`, `useUndo`, `useWaitForJob`, listener e helpers de
  plataforma dos atalhos, `wingPathParam`, `deriveTitle` e `explorer/utils`.

Medição local com o mesmo script e o mesmo relatório (bun local): interface 4.923/6.010 linhas
(81,9%), os mesmos 73 arquivos. Os módulos-alvo ficaram entre 89% e 100%. O total de linhas
instrumentadas difere do CI por versão do bun; o número oficial é o do workflow Coverage após o merge.

Falta: copiar/recortar/colar e exclusão pela UI, labels acessíveis e catálogos en/pt, e o smoke de
Explorer/sidebar com web UI e daemon isolado. Maiores déficits restantes: `TabBar.tsx`,
`WebContextMenuContext.tsx`, `useContextMenu.ts` e `IntegrationsSettings.tsx`.

## Measurement 2026-11-15

Coverage workflow of `main` at `5c7764815` (run 43686539472), `summary.json` on the `badges`
branch and the `coverage-rust` LCOV artifact of the same run:

| Area | Lines |
|---|---|
| Core | 53.7% (48,548 / 90,473) |
| Desktop (Tauri shell and its crates) | 90.7% (7,842 / 8,647) |
| CLI | 65.9% (4,830 / 7,330) |
| Interface (React) | 74.5% (5,159 / 6,925) |
| `core/src/ops/files` | 63.9% (5,901 / 9,234) |
| `core/src/ops/indexing` | 73.7% (6,567 / 8,909) |
| `core/src/infra/db` | 54.6% (3,020 / 5,529) |
| `core/src/domain/addressing` | 76.8% (553 / 720) |
| `core/src/infra/wire` and `infra/api` | 80.3% (1,174 / 1,462) |
| `core/src/infra/job` | 43.1% (1,294 / 2,999) |
| Sync (`service/sync`, `infra/sync`) | 72.5% (5,885 / 8,121) |

The four M3 component goals (core 50%, desktop 75%, CLI 60%, interface 60%) are met on `main`.
Moved to M3 with the rest of the coverage epic, since its open criteria are module goals and gates,
not M2 work.

The interface is at 74.5%.
