---
id: COV-09
title: "Cobrir parsing, configuração e saída do CLI"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, cli]
last_updated: 2026-11-11
---

## Description

O CLI parte de 0,4%; parsing e persistência precisam de testes que cubram comandos e erros sem depender de serviços pessoais.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Expandir a suíte inicial para todos os domínios Clap, aliases, flags e valores inválidos.
  Feito para busca, arquivos, indexação, jobs, tags, volumes, locations, rede, redundância,
  eventos, sync e logs. Faltam `cloud setup` (0%), `daemon` e `confirm`.
- [x] Cobrir seleção de biblioteca, migração de configuração, erros e saída human/JSON.
- [x] Testar resolução de caminhos e conversão de args para os inputs reais do core.

## Acceptance Criteria

- [x] CLI atinge pelo menos 25% na primeira etapa; parsing/configuração selecionados atingem 85%.
- [ ] JSON é validado estruturalmente e erros preservam contexto útil. Os erros estão cobertos;
  a saída JSON roda em dois testes, mas o stdout não é capturado nem validado.
- [x] Fixtures temporárias e todos os testes de wing-cli executados pelo gate local e CI.

## Evidence 2026-11-11

Ambiente: Linux x86_64, `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo-llvm-cov 0.9.1`, branch
`test/cli-coverage`, `target/` aquecido do checkout principal.

A suíte do binário `wing` passou de 28 para 513 testes. Os 485 novos se dividem em:

| Tipo | Onde | O que verificam |
|---|---|---|
| Parser e conversão | `#[cfg(test)]` em cada `domains/*/args.rs`, `config/mod.rs`, `ui/progress.rs` | Padrões, flags, valores inválidos e o input do core gerado por cada comando |
| Matrizes de combinações | `domains/*/args/matrix.rs` | Combinações de opções de busca, cópia, indexação, eventos, tags, comparação de volumes e importação |
| Dispatcher com RPC sintético | `src/tests/rpc.rs` | Método, biblioteca e payload enviados, saída human, erros do daemon e códigos de retorno |
| Árvore de comandos e configuração | `src/tests/mod.rs` | `Cli::command().debug_assert()`, nomes de instância, migração e preservação do `cli.json`, reset |
| Conflito de cópia e logs | `domains/file/mod.rs`, `domains/logs/mod.rs` | Destinos livres e ocupados, link quebrado, destino remoto, diretório de dados por instância |

O RPC sintético escuta em `127.0.0.1:0`, responde a uma sequência fixa de requests e verifica cada
um; não há daemon real, rede externa nem acesso ao HOME. Fixtures de arquivos e configuração usam
`tempfile`. As matrizes cobrem parsing e conversão, não fluxos de ponta a ponta.

Os testes acharam 16 bugs, todos corrigidos e descritos em [BUGS.md](BUGS.md) (entradas "CLI:").
Os de maior impacto: a busca com `--wing-path` inválido pesquisava a biblioteca inteira, a indexação
mandava `content://invalid` ao core como caminho local, a checagem de conflito da cópia só olhava a
primeira origem e `job pause`, `resume` e `cancel` saíam com código 0 quando falhavam.

| Comando | Resultado |
|---|---|
| `cargo test --locked -p wing-cli --bin wing` | 513 aprovados, 0 falhas, 0 ignorados; testes em 0,03 s |
| `cargo clippy --locked -p wing-cli --all-targets -- -D warnings` | Aprovado, código 0 |
| `cargo llvm-cov --locked -p wing-cli --bin wing --lcov --output-path target/cov-cli/cli.lcov` | Aprovado, 513 testes |
| `python3 scripts/coverage-cli-production.py target/cov-cli/cli.lcov` | 3.016 de 5.185 linhas, 58,2% |

Cobertura de linhas do CLI, só com a suíte do `wing`:

| Medida | Linhas cobertas/instrumentadas | Percentual |
|---|---:|---:|
| Referência no `main` (`summary.json` do branch `badges`, SHA `27e8546fe`) | 449/5.384 | 8,3% |
| Produção, sem módulos `#[cfg(test)]` nem matrizes (`coverage-cli-production.py`) | 3.016/5.185 | 58,2% |
| Mesmo filtro do relatório completo (`coverage-report.py`) | 4.830/7.001 | 69,0% |
| `args.rs` de todos os domínios mais `config.rs` e `domains/config/mod.rs` | 601/614 | 97,9% |

O relatório completo conta como produção as linhas dos módulos `#[cfg(test)]` e das matrizes, que
sempre executam, por isso fica acima da medida de produção. A meta da primeira etapa (25%) e o piso
de parsing/configuração (85%) foram medidos pela linha de produção. O badge do `main` vai refletir o
número do relatório completo depois do merge.

Arquivos de produção ainda abaixo de 25%: `domains/cloud/setup.rs` (0%), `domains/daemon/mod.rs`
(3,8%), `util/confirm.rs` (3,4%), `util/macros.rs` (3,6%), `ui/colors.rs` (23,1%) e
`domains/search/mod.rs` (24,0%). O fluxo interativo, a execução contra daemon real e o Windows
ficam com a COV-10; a saída JSON validada por estrutura continua aberta nesta task.
