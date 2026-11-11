---
id: COV-06
title: "Cobrir endereçamento, validação e contratos RPC"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, core]
last_updated: 2026-11-11
---

## Description

Caminhos locais, remotos e de conteúdo e o registro Wire sustentam operações usadas por todos os clientes.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Cobrir WingPath e batches, URIs inválidas, Unicode e resolução sem seguir links.
- [ ] Testar registro de queries/actions e erros de método, payload e biblioteca ausente.
- [ ] Testar formatos públicos consumidos pelo ts-client sem redefinir tipos.

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

