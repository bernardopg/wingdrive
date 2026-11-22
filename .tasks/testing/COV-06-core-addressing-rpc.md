---
id: COV-06
title: "Cobrir endereçamento, validação e contratos RPC"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, core]
last_updated: 2026-11-22
---

## Description

Caminhos locais, remotos e de conteúdo e o registro Wire sustentam operações usadas por todos os clientes.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Cobrir WingPath e batches, URIs inválidas, Unicode e resolução sem seguir links.
- [x] Testar registro de queries/actions e erros de método, payload e biblioteca ausente.
- [x] Testar formatos públicos consumidos pelo ts-client sem redefinir tipos.

## Acceptance Criteria

- [ ] Ao menos 85% de linhas nos módulos selecionados de endereçamento e validação.
- [ ] Erros de protocolo e de domínio mantêm códigos e payloads esperados.
- [ ] Tipos públicos continuam atuais após generate_typescript_types.

## Evidence 2026-11-10

Ambiente: Linux x86_64, `rustc 1.99.0 (b940084d7 2026-09-28)`, branch
`t3/cover-core-validation-indexing`, target exclusivo em `$PWD/target`, builds sequenciais com `-j 6`.
Commit desta tranche: `b4e1007296f5568e70976b18c3b36ea1873753b4`.
Validação conjunta com COV-04 no commit `2db175935db6d349ebd59797bd0e7a8ac745354c`.

Foram adicionados 31 testes:

| Módulo ou contrato | Novos testes |
|---|---:|
| WingPath, batches, sidecars e provedores de nuvem, em `addressing_contract_test.rs` | 8 |
| `ops/files/rename/validation.rs` | 2 |
| `ops/files/create_entry/mod.rs` | 5 |
| `ops/files/compare.rs` | 4 |
| `infra/api/error.rs` | 2 |
| `infra/api/permissions.rs` | 5 |
| `infra/wire/registry.rs`, incluindo extração de tipos e escopos | 2 |
| Dispatcher, handlers registrados e `files.createFolder`, em `rpc_validation_contract_test.rs` | 3 |

Os casos incluem as 42 combinações de tipo/formato de sidecar, os dez provedores de nuvem,
Unicode, nomes fora de UTF-8 no Unix, links quebrados, conflitos de criação, nomes inválidos,
permissões, payload escalar inválido em cada handler registrado, método desconhecido,
biblioteca não selecionada ou inexistente e formato público dos erros e resultados.
O harness com watcher desativado usa diretório temporário, sem criar fixtures em HOME.
A nova integração RPC está na lista Linux de `scripts/ci-rust.sh`, usada também pela cobertura.

| Comando exato | Resultado |
|---|---|
| `CARGO_TARGET_DIR=$PWD/target cargo fmt --all -- --check` | Aprovado, código 0, nenhuma diferença |
| `CARGO_TARGET_DIR=$PWD/target cargo clippy --locked -j 6 -p wing-core -- -D warnings` | Aprovado, código 0; 3 min 42 s |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --lib` | 532 aprovados, 0 falhas, 2 ignorados preexistentes; build 4 min 14 s, testes 1,40 s |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --test addressing_contract_test --test indexing_rules_test --test rpc_validation_contract_test` | Endereçamento 15/0/0, regras 10/0/0, RPC 5/0/0, na ordem aprovados/falhas/ignorados; build 4 min 07 s |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --test rpc_validation_contract_test` | Reexecução após ampliar o caso de biblioteca ausente: 5 aprovados, 0 falhas, 0 ignorados; build 2,27 s, testes 11,15 s |
| `bash -n scripts/ci-rust.sh` | Aprovado, código 0 |

O arquivo RPC executa três testes novos e dois testes já existentes dos helpers.
Antes das correções, `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --test addressing_contract_test`
registrou 12 aprovados e 3 falhas: URI de tipo de sidecar ausente, JSON MessagePack e esquema
`cloud`. Os três defeitos foram corrigidos e descritos em [BUGS.md](BUGS.md).
Nenhum teste novo foi ignorado; os dois ignorados da suíte unitária são o benchmark de 100 mil
entradas e a integração S3 já existentes.

Limites: não houve execução em Windows nem medição LCOV ou de linhas cobertas/instrumentadas.
Esta evidência não comprova a meta de 85%. Os formatos dos tipos públicos foram preservados;
`generate_typescript_types` não foi executado. Espaços, tags, permissões de filesystem e
validações de locations ainda precisam de novas tranches. A task passou a In Progress com o merge do #359; os critérios seguem abertos até a medição LCOV.

## Evidence 2026-11-11

Ambiente: Linux x86_64, `rustc 1.99.0`, `cargo-llvm-cov 0.9.1`, branch `test/core-coverage-1`.

Primeira tranche do core rumo a 80% de linhas. A triagem rodou as 42 suítes de `core/tests/` que o
CI não executava; as que passaram e as três novas entraram na lista Linux de `scripts/ci-rust.sh`
depois de três execuções seguidas sem falha.

| Suíte | Situação |
|---|---|
| `file_queries_test` (nova) | 9 testes: `files.media_listing`, `files.by_path`, `files.by_id`, `files.alternate_instances`, `tags.files_by_tag` e resolução de caminho |
| `index_verify_test` (nova) | 3 testes de `indexing.verify`: índice novo, mudanças por fora do índice e pasta fora de location |
| `space_layout_test` (nova) | 2 testes: layout com grupos e itens fixados, reordenação, renomeação, exclusão e erros |
| `file_sync_test` | Falhava em 9 de 10 casos com `no such table: main.entry`; 10/10 após a correção da migração |
| `folder_rename_test`, `copy_progress_test`, `config_persistence_test`, `database_migration_test`, `event_system_test`, `job_shutdown_test`, `normalized_cache_fixtures_test`, `indexing_responder_reindex_test`, `proxy_pairing_protocol_test`, `analyze_events` | Passavam e não rodavam no CI |

Fora do CI, com o motivo: `aggregate_sizes_watcher_test`, `ephemeral_watcher_test`, `fs_watcher_test`
e `file_move_test` passam, mas dependem do watcher e não passaram pelas três rodadas.
`job_resumption_integration_test` depende de tempo (a indexação passou da fase antes da
interrupção). `phase_snapshot_test` e `resource_events_test` exigem `~/Desktop`.
`volume_tracking_test` espera um volume `swap`. `wasm_extension_test` e `wasm_job_execution_test`
carregam um `.wasm` versionado compilado com o módulo de import antigo `spacedrive`; o SDK atual
usa `wingdrive`. `ephemeral_bridge_test` e `typescript_bridge_test` precisam de `node_modules` no
checkout. Os testes de pareamento, cópia e transferência entre dispositivos passam, mas levam cerca
de dois minutos cada; `transitive_sync_backfill_test` estourou o tempo de espera. Esses ficam para
a COV-05.

Bugs corrigidos (detalhes em [BUGS.md](BUGS.md), entradas "Core:"): grupos e itens de Space enviados a outros dispositivos com o ID local do pai, achado na revisão do PR; `has_more` da listagem de mídia
nunca verdadeiro e `_`/`%` tratados como curinga; resolução de caminho sem achar pasta com ponto e
escolhendo location por prefixo de string; `indexing.verify` acusando a própria pasta e os
ancestrais; `tags.files_by_tag` com erro só quando `include_children`; exclusões e reordenações de
Spaces sem sync; item de Space em grupo de outro espaço; item fixado resolvendo o arquivo errado;
`sync_conduit` com chaves estrangeiras para a tabela inexistente `entry`. A revisão do PR também
apontou o placeholder `local` e locations sem volume na resolução de caminho, itens de dois espaços
aceitos juntos na reordenação, exclusão apagada antes do registro de sync, reordenação gravada pela
metade, a pasta verificada fora da comparação do `indexing.verify` e a migração sem transação; todos
foram corrigidos com teste. As suítes novas também entraram em `cargo xtask test-core`.
Inconsistências removidas: `ops::addressing::PathResolver`, alcançável só por `WingPath::resolve`,
que nada chamava; um comentário de documentação de `filters.rs` estava sobre a função errada.

| Comando | Resultado |
|---|---|
| `cargo llvm-cov` com `wing-core --lib` e as suítes de `scripts/ci-rust.sh`, após `cargo llvm-cov clean --workspace` | 821 testes aprovados, 0 falhas |
| Componente `core/` com o filtro de `scripts/coverage-report.py` | 42.582/83.586 linhas (50,9%); o relatório do `main` tem 39.137/83.573 (46,8%) |
| As 14 suítes adicionadas ao CI, três rodadas seguidas | Sem falha |
| `cargo clippy --locked -p wing-core -- -D warnings` | Limpo |
| Testes novos contra o código antigo | `has_more`, curinga de pasta e pasta com ponto falham antes da correção |

Maiores ganhos por arquivo, em linhas: `media_listing.rs` 0 para 277, `files_by_tag.rs` 0 para 233,
`domain/space.rs` 152 para 373, `verify/action.rs` 0 para 210, `service/file_sync/mod.rs` 0 para
165, `file_by_path.rs` 0 para 157, `alternate_instances.rs` 0 para 147, `spaces/get_layout` 0 para 133.

Limites: a medição local não inclui os testes de `wing-server` e dos crates compartilhados que o
workflow de cobertura também roda, então o número do CI pode ser um pouco maior. A replicação dos
Spaces entre dois dispositivos não foi exercitada. Não houve execução em Windows nem `just ci-local`.

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

Addressing is at 76.8%, below the 85% goal.

## Evidence 2026-11-22

Ambiente: Linux x86_64, worktree `/home/bitter/.cache/opencode/wt/cov-06` (branch `test/cov-06-addressing-rpc`, base `origin/main`), `rustc`/cargo do toolchain local. Não houve alteração de tipos públicos; `cargo run --bin generate_typescript_types` não foi executado.

Testes adicionados/ajustados:

| Área | Cobertura adicionada |
|---|---|
| `core/tests/addressing_contract_test.rs` | Casos de URI inválida/Unicode (`content://`, extra em content, scheme desconhecido, sidecar com segmento extra, `local://` vazio), acessores/display de todas as variantes de `WingPath`, estabilidade de `WingPathParseError` display, comportamento de `WingPathBatch` (default, vazio, local/cloud filtering, agrupamento por device), e paridade `from_uri_with_context` com `from_uri` via harness isolado. |
| `core/src/domain/addressing.rs` (módulo de testes) | Display/From de `PathResolutionError` e `WingPathParseError`, `WingPathBatch::default/local_only`, panics de `join` para `Content` e `Sidecar`. |

Comandos executados:

| Comando exato | Resultado |
|---|---|
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -p wing-core --lib domain::addressing -j 4` | Falhou durante compilação: `rustc` SIGTERM (signal 15) ao compilar `cranelift-codegen` em `target/debug/build/...`; sem resultado de testes. |
| `CARGO_TARGET_DIR=/home/bitter/development/wingdrive/target cargo test --locked -p wing-core --lib domain::addressing -j 4` | Falhou durante compilação: `rustc` SIGTERM (signal 15) ao compilar `wing-core` (lib test build) com target compartilhado; sem resultado de testes. |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -p wing-core --test addressing_contract_test` | Falhou durante compilação: `rustc` SIGTERM (signal 15) ao compilar `cranelift-codegen`; sem resultado de testes. |
| `CARGO_TARGET_DIR=/home/bitter/development/wingdrive/target cargo test --locked -p wing-core --test addressing_contract_test -j 2` | Falhou durante compilação: `rustc` SIGTERM (signal 15) ao compilar `wing-core` (test build) com target compartilhado; sem resultado de testes. |
| `cargo fmt -p wing-core` | Aprovado, código 0. |

Lacunas: execução de testes Cargo bloqueada por OOM/SIGTERM no ambiente ao compilar `wing-core`; não foi possível confirmar passagem dos novos testes nem medir cobertura LCOV. Critério de 85% de linhas continua sem comprovação. Registro/erros RPC e formatos públicos já tinham evidência anterior; esta tranche apenas reforça casos de addressing/URI/Unicode/batch sem alterar tipos públicos.
