---
id: COV-04
title: "Cobrir banco, indexação e reconciliação do watcher"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
sprint: S05
tags: [coverage, testing, core]
last_updated: 2026-11-10
---

## Description

Listagens e metadados devem permanecer consistentes quando o filesystem muda durante a indexação.

## Dependencies

COV-03. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Testar migrations em SQLite temporário, rollback e reabertura.
- [ ] Testar rename, exclusão, subtree, hidden files, symlinks e reconciliação de UUID.
- [ ] Testar eventos duplicados e concorrentes com fixtures limitadas e waits por condição.

## Acceptance Criteria

- [ ] Ao menos 70% de linhas nos módulos de indexação e persistência selecionados no inventário.
- [ ] Sem fixtures dependentes do checkout, da biblioteca do autor ou de sleeps fixos.
- [ ] Segunda execução de indexação não duplica registros nem perde tags.

## Evidence 2026-11-10

Ambiente: Linux x86_64, `rustc 1.99.0 (b940084d7 2026-09-28)`, branch
`t3/cover-core-validation-indexing`, target exclusivo em `$PWD/target`, builds sequenciais com `-j 6`.
Commit desta tranche: `2db175935db6d349ebd59797bd0e7a8ac745354c`, após a tranche COV-06
`b4e1007296f5568e70976b18c3b36ea1873753b4`.

Foram adicionados 12 testes:

| Módulo ou contrato | Novos testes | Casos |
|---|---:|---|
| `infra/db/mod.rs` | 5 | Migrations idempotentes e reabertura, rollback de schema e dados, falha de migration sem mudanças parciais, escritores concorrentes e abertura de banco ausente |
| `ops/indexing/rules.rs`, via `indexing_rules_test.rs` | 7 | Rejeição por diretórios filhos, erros de IO, serialização JSON/MessagePack, globs inválidos, clone do ruler, nomes fora de UTF-8, links e limites das regras Git |

Todos os bancos e diretórios novos são descartáveis. A coordenação dos escritores usa canal
oneshot e timeout, sem sleeps. Nenhum teste novo acessa rede ou lixeira.

| Comando exato | Resultado |
|---|---|
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --test indexing_rules_test` | Antes da correção: 8 aprovados, 2 falhas, 0 ignorados |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --test addressing_contract_test --test indexing_rules_test --test rpc_validation_contract_test` | Após a correção: regras 10 aprovados, 0 falhas, 0 ignorados; endereçamento 15/0/0 e RPC 5/0/0; build conjunto 4 min 07 s |
| `CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wing-core --lib` | 532 aprovados, 0 falhas, 2 ignorados preexistentes; inclui os cinco testes novos de banco; build 4 min 14 s, testes 1,40 s |
| `CARGO_TARGET_DIR=$PWD/target cargo clippy --locked -j 6 -p wing-core -- -D warnings` | Aprovado, código 0; 3 min 42 s |
| `CARGO_TARGET_DIR=$PWD/target cargo fmt --all -- --check` | Aprovado, código 0, nenhuma diferença |

As duas falhas reproduzidas eram a aceitação de uma pasta que deveria ser rejeitada pela regra
de filhos e a conversão de erro de `read_dir` em erro de MessagePack. As correções e seus limites
estão em [BUGS.md](BUGS.md). Nenhum teste novo foi ignorado.

Limites: não houve execução em Windows nem medição LCOV ou de linhas cobertas/instrumentadas;
a meta de 70% permanece sem comprovação. A prioridade foi COV-06 e os gates obrigatórios.
Não foram ampliados nesta tranche os testes de downgrade de migrations, reconciliação de UUID,
watcher com eventos duplicados/concorrentes, preservação de tags na segunda indexação ou
invariantes da closure table após move/rename/delete. A task passou a In Progress com o merge do #359; os critérios seguem abertos até a medição LCOV.
