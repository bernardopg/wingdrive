---
id: COV-09
title: "Cobrir parsing, configuração e saída do CLI"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, cli]
last_updated: 2026-11-23
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

The CLI is at 65.9%.

## EOF em prompts interativos

A leitura de EOF nos prompts de escolha e de texto obrigatório repetia até o processo ser interrompido. Veja [o diagnóstico em BUGS.md](BUGS.md#prompts-do-cli-repetem-indefinidamente-com-stdin-fechado). Casos de teste devem verificar encerramento, entrada inválida seguida de escolha válida, opção vazia e erro de I/O.

A correção retorna erro em EOF, preserva novas tentativas em entrada inválida e rejeita
listas sem opções. Cinco testes dos prompts passaram no harness isolado. A execução de
processo repetiu os três casos originais e terminou com código 1 em todos. As unidades
ficam na suíte do binário `wing` e a medição completa do componente permanece pendente.
O primeiro build do harness tentou reutilizar dependências de outro toolchain e falhou
com E0514. A repetição usou `anyhow` do rustc atual e as funções sem suporte a TTY.

## Cloud sync schedule regression 2026-11-19

The CLI loopback fixture confirmed that `integrations cloud-sync update <id> --every 0` sent
`null` in the old build. A separate test called the registered RPC handler with a disposable
SQLite library. It confirmed that `null` preserved the interval and that zero failed validation
before the correction. Both baseline failures are retained in
`coverage/goal/cloud-sync-cli-before.log` and `coverage/goal/cloud-sync-before.log`.
The baseline core log also contains expected sync event debug traces for the temporary library.
The protocol diagnosis and correction are recorded in [BUGS.md](BUGS.md).

After correction, the instrumented CLI suite passed 562 tests and
`rpc_validation_contract_test` passed six, with no failures, ignored tests or compiler warnings.
The Linux type generator ran and only changed the documentation on the generated field.
The first correction build had two E0308 errors and one E0605 error because the field still used
the nested option. Replacing the field type resolved all three errors. The intermediate compiler
diagnostics are in `coverage/goal/cloud-sync-compile.log`; the successful run and generator output
are in `coverage/goal/cloud-sync-after.log`.

## Evidence 2026-11-22 (cov-09 worktree)

Branch: `test/cov-09-cli-contracts` (worktree `/home/bitter/.cache/opencode/wt/cov-09`, based on `origin/main`).

### Changes

- `apps/cli/src/tests/mod.rs`: added parsing/dispatch contract tests for `cloud` (rejects extra args/unknown shape), `daemon` subcommands (install/uninstall/status parse and invalid shapes), and top-level aliases/shapes (`wings`/`spaces`, legacy `network spacedrop` alias, invalid global flag positions).
- `apps/cli/src/domains/daemon/mod.rs`: expanded `qualified_service_name`/`systemd_service_name` default-instance assertions and added safe `service_active` unknown-unit test (returns false; no system mutation).
- `apps/cli/tests/cli_process.rs`: added reusable JSON RPC fixture helper; added structural JSON stdout validation for `tag search --format json` (asserts JSON shape/fields and request payload); added JSON error-path process test asserting daemon validation error context is preserved on stderr and stdout has no fake success payload; added `stop --reset` process test with `WING_CLI_YES=1` on isolated temp data dir/instance verifying confirmation bypass and safe v2-only reset.

### Validation

| Command | Result |
|---|---|
| `cargo fmt -p wing-cli` | Passed. |
| `cargo test -p wing-cli --bin wing` | **Not completed in this environment**: 3 attempts in cov-09 were SIGTERM'd after 900s during compilation before test execution. |
| `cargo test -p wing-cli --test cli_process` | Not run separately (compile did not complete). |
| Coverage (`cargo llvm-cov` / `scripts/coverage-cli-production.py`) | Not run; compile-time blocked, not cheap in this env. |

### Gaps / honesty

- Task-named gaps `cloud setup`, `daemon`, and `confirm` received targeted contract/process coverage additions, but fresh line-coverage numbers were not produced due environment compile-time SIGTERM.
- `daemon install/uninstall/status` runtime paths remain OS/systemd-interactive; tests avoid mutating real user services. Deeper runtime coverage needs isolated system fixtures or a warm CI environment.
- JSON structural validation improved by process tests, but is not yet comprehensive across all JSON-producing commands.

## Realignment (2026-11-23)

- PR #504 merged with CLI parsing/JSON/error-context tests. Coverage not measured due compile SIGTERM; daemon install/uninstall runtime coverage limited. Remains In Progress.
