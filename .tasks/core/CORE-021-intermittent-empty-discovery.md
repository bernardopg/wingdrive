---
id: CORE-021
title: Discovery intermitente encontra zero itens numa pasta com arquivos
status: To Do
assignee: bernardopg
parent: CORE-000
priority: High
tags: [core, indexing, bug, flaky]
milestone: M2
sprint: S06
last_updated: 2026-11-13
---

## Description

Duas vezes em 2026-11-13 a fase de discovery do indexer terminou com
"Parallel discovery complete: 0 files, 0 dirs, 0 symlinks, 0 skipped" numa
pasta recém-criada com arquivos. Nos dois casos a re-execução passou e nenhum
código de indexação mudou desde a `v3.1.1`.

- CI do #388, `normalized_cache_fixtures_test::capture_event_fixtures_for_typescript`:
  location com 5 arquivos e 2 subpastas, indexação persistente, terminou com
  `Indexing timeout after 30s (entries: 1)`.
- `release.yml` da `v3.2.0` (run 43392546833, tentativa 1), smoke
  `scripts/release/explorer.cjs`: a pasta com 10.002 itens abriu no Explorer
  como "0 files" e o Playwright esperou 90 s por `item-00000.txt`. Havia um
  índice efêmero de `/tmp`, pai da pasta, criado antes.

## Implementation Steps

- [ ] Reproduzir em laço: rodar o teste de fixtures e o smoke do Explorer
      repetidas vezes no Linux, com `RUST_LOG=wing_core::ops::indexing=trace`.
- [ ] Verificar se o backend de volume (`Root`) ou o `read_dir` devolvem vazio
      sem erro, e se a listagem efêmera reaproveita o índice do pai em vez de
      indexar a pasta.
- [ ] Corrigir a causa e cobrir com teste de regressão.

## Acceptance Criteria

- Discovery de uma pasta com itens nunca termina com zero itens sem reportar erro.
- 50 execuções seguidas do teste de fixtures e 10 do smoke do Explorer passam.
