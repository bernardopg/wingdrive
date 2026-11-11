---
id: COV-06
title: "Cobrir endereçamento, validação e contratos RPC"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, core]
last_updated: 2026-11-10
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
