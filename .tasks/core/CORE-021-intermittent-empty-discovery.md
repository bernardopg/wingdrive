---
id: CORE-021
title: Discovery intermitente encontra zero itens numa pasta com arquivos
status: Done
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

- [x] Reproduzir em laço: rodar o teste de fixtures e o smoke do Explorer
      repetidas vezes no Linux, com `RUST_LOG=wing_core::ops::indexing=trace`. (fixture
      reproduzido; smoke não reproduziu em 15 runs, 2026-11-14/15)
- [x] Verificar se o backend de volume (`Root`) ou o `read_dir` devolvem vazio
      sem erro, e se a listagem efêmera reaproveita o índice do pai em vez de
      indexar a pasta. (causa: location dentro do data dir do Core; caminho efêmero não
      explicado, mas não reproduzido)
- [x] Corrigir a causa e cobrir com teste de regressão. (indexação persistente falha com
      erro claro; `core/tests/own_data_location_test.rs`)

## Acceptance Criteria

- [x] Discovery de uma pasta com itens nunca termina com zero itens sem reportar erro.
      (location dentro do data dir agora falha com erro explícito; smoke não reproduziu o vazio em 15 runs)
- [x] 50 execuções seguidas do teste de fixtures (feito em 2026-11-14) e 10 do smoke do Explorer passam.

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

## Explorer smoke, 2026-11-15

Release `wing-server` built from `main` (`5c7764815`) with `--features wing-core/ffmpeg,wing-core/heif`,
isolated data dir, Playwright 1.62.1 and `scripts/release/explorer.cjs` unchanged:

- 10 runs in a row against one server: 10 passed (18 to 26 s each).
- 5 more runs that open `/tmp` in the Explorer first, so the ephemeral index of the fixture's parent
  exists before the fixture (the lead from the `v3.2.0` failure): 5 passed.
- No `Parallel discovery complete: 0 files, 0 dirs` line in either server log.

The `v3.2.0` failure did not come back in 15 runs, and the fixture-test causes were fixed on
2026-11-14, so the acceptance criteria are met. TAURI-018 runs the smoke on every pull request
and 10 times each night, which measures the failure if it still exists. The warnings the server
logged during these runs went to CORE-022.
