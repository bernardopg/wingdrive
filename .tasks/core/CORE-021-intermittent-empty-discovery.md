---
id: CORE-021
title: Discovery intermitente encontra zero itens numa pasta com arquivos
status: In Progress
assignee: bernardopg
parent: CORE-000
priority: High
tags: [core, indexing, bug, flaky]
milestone: M2
sprint: S05
last_updated: 2026-11-15
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
- 50 execuções seguidas do teste de fixtures (feito em 2026-11-14) e 10 do smoke do Explorer passam.

## Investigation (2026-11-14)

**Fixture test: two causes found.** `normalized_cache_fixtures_test` created the Core with
`temp_dir` as its data dir and the location at `temp_dir/test_location`, inside it. Since e52b16b36
(2026-10-05) the walk drops everything under the daemon's own data dir (`config::is_own_data`, so
the daemon never indexes its own SQLite journals), so this location always finished discovery with
`0 files, 0 dirs, 0 symlinks, 0 skipped` and no error: only the root entry existed. That part was
deterministic. The intermittent failure came from `wait_for_indexing_completion`, which only counted
the job as seen once it caught it in `Running`; a job with nothing to walk often finished between two
polls, and the wait ran into its 30 s timeout with `entries: 1`.

- Persistent indexing now fails with `<path> is inside WingDrive's own data folder, which is never
  indexed` instead of finishing empty (`ops/indexing/job.rs`).
- `normalized_cache_fixtures_test`, `file_structure_test` and `event_system_test` keep the Core's
  data dir in `temp_dir/data`, beside their locations, and the fixture wait counts a job it finds
  already completed.
- Regression test `core/tests/own_data_location_test.rs`, alone in its binary so its Core is the
  first: a location inside the data dir fails with that message, and one beside it indexes.
- `normalized_cache_fixtures_test` binary run 50 times in a row on 2026-11-14: 50 pass, 0 fail.

**Explorer smoke: not explained by this.** The release smoke runs `wing-server --data-dir
/tmp/wingdrive-browser` and the fixture is `/tmp/wingdrive-explorer-*`, outside it. The ephemeral
index of `/tmp` noted above stays the lead for that failure.

