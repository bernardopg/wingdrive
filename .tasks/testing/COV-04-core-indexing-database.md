---
id: COV-04
title: "Cobrir banco, indexação e reconciliação do watcher"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, core]
last_updated: 2026-11-15
---

## Description

Listagens e metadados devem permanecer consistentes quando o filesystem muda durante a indexação.

## Regressão de eventos em diretório temporário

O watcher efêmero expirava após criar `document.txt` sob uma raiz temporária com
`.tmp` no nome. O log `coverage/goal/after-sidecars/watchers.log` registra a inscrição
bem-sucedida, a criação e a ausência de eventos. O filtro agora compara o nome da entrada,
e a fixture isola os arquivos sem alterar o `HOME`. Detalhes em [BUGS.md](BUGS.md).
Os 31 testes instrumentados do crate de watcher e as duas suítes de integração passaram.
O boot temporário registrou a criação da configuração padrão e o fallback de identificação
de volumes sem permissão de escrita. A suíte persistente registrou uma colisão concorrente
de MIME, registrada separadamente no histórico de bugs.

Dois escritores de MIME agora reutilizam a mesma linha SQLite. A regressão concorrente e
o replay instrumentado do watcher persistente passaram. O replay criou a configuração
padrão e registrou o fallback de identificação dos volumes protegidos. Também emitiu
uma ocorrência não crítica de `ENOENT` ao calcular o hash do arquivo que o cenário de
extração exclui durante a indexação. O teste confirmou a exclusão. Os logs estão em
`coverage/goal/after-sidecars/mime-fix.log` e `persistent-watcher-mime-fix.log`.
O primeiro build do patch denunciou um delimitador Rust faltante, corrigido antes do replay.
A tentativa com `RETURNING` foi interrompida ao confirmar que o ORM trata zero linhas como
`RecordNotFound`. A versão validada usa a contagem de linhas do insert.



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

Indexing is at 73.7%, above the 70% goal; persistence (`infra/db`) is at 54.6%.

## Evidence 2026-11-19

The Linux Rust gate explicitly enumerates integration suites. Five existing suites were absent from
that list and consequently from the official Rust coverage collector. The gate now runs them.

| Suite | Passed | Behavior exercised |
|---|---:|---|
| `aggregate_sizes_watcher_test` | 3 | Watcher updates to aggregate directory sizes |
| `google_drive_picker_test` | 1 | Drive Picker integration with a local test server |
| `microsoft_integration_test` | 1 | Microsoft integration with a local test server |
| `uuid_reconciliation_test` | 6 | Persistent and ephemeral UUID identity, tags and library isolation |
| `volume_offline_test` | 1 | Per-device offline state |

All 12 passed on Linux with LLVM instrumentation and the collector's Cargo package selection.
The focused run had zero failures, ignored tests or compiler warnings. The full report must be
regenerated before attributing any percentage gain to these suites. The collector now invokes
`cargo llvm-cov show-env --sh` to remove the deprecated `--export-prefix` warning.

### Ponte de eventos Rust e TypeScript

A suíte de cache usa o cliente real por TCP e verifica eventos do watcher depois de
movimentar, renomear e excluir arquivos descartáveis. O primeiro replay registrou quatro
falhas de fixture por interceptar a API privada de assinaturas. Depois de remover essa
interceptação, restaram as duas falhas de movimentação, que preservavam entradas na origem.
A correção encaminha o evento aos dois escopos. Ver [BUGS.md](BUGS.md).

O replay completo da ponte de cache passou seis testes no Linux com LLVM. Outros 15
testes de busca, eventos efêmeros e filtros passaram no mesmo checkout. Os logs
`coverage/goal/after-watchers/typescript-bridge-routing.log` e
`bridge-routing-neighbors.log` não registraram avisos de compilação. As três suítes de
ponte entram no CI Linux e no coletor. As duas configurações passaram no `actionlint`.

### GitHub Actions · promoção de arquivo observada no PR #474

O log completo do job confirmou a entrega do evento e uma inferência de move incorreta por inode.
O teste de watcher deve indexar o arquivo promovido sem mover nem substituir a raiz da localização.
Veja [registro de bugs](BUGS.md#core-reutilização-de-inode-quebrava-a-promoção-de-um-arquivo-no-watcher).

### Offline sidecar recovery

A temporary library fixture verifies sidecar discovery from disk, the ready lookup, database
presence, size updates, local availability after removal and the uninitialized library error.
The instrumented Linux test passed. It hit 365 of 569 lines in `sidecar_manager.rs`. The prior
complete profile hit 20 of 569. These are per-file profiles. The next complete collector measures
the combined core impact. The test requires no external services or media decoder.
