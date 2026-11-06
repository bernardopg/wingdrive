---
id: COV-02
title: "Adicionar a primeira suíte de regressões de contratos"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, ci]
last_updated: 2026-11-06
---

## Description

Proteger identidade de conteúdo, argumentos e persistência do CLI e abertura de caminhos no desktop com fixtures isoladas.

## Dependencies

COV-01. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Testar hashing pequeno, amostrado, integridade completa, mutação e erros de I/O.
- [x] Testar argumentos destrutivos, instâncias, configuração legada/corrompida e persistência do CLI.
- [x] Testar URIs locais, Unicode, links simbólicos e consumo de início oculto no desktop.
- [x] Integrar os novos grupos aos runners de CI e de cobertura.

## Acceptance Criteria

- [x] Testes verificam resultados e preservação de dados, sem daemon pessoal, rede externa ou credenciais.
- [x] Core, desktop e CLI passam localmente e falhas propagam para o gate.
- [ ] Evidências registram comandos e limites da validação; DoD exige merge no main.

## Evidence

- CLI: `cargo test --offline -p wing-cli --bin wing`, 18 passed, 0 failed, sendo 16 testes novos.
- Suites novas em `core/src/domain/content_identity/tests/`, `apps/cli/src/tests/` e
  `apps/tauri/src-tauri/src/launch/tests/`; diretórios de testes excluídos do agregador LCOV.
- `scripts/ci-rust.sh` executa o CLI no gate completo e no escopo `cli`; o grupo de identidade
  do core entra no gate completo. Os casos de launch rodam na suíte desktop existente.
- `scripts/coverage-rust.sh` reutiliza esse runner e evita executar o CLI duas vezes.
- A porcentagem completa após estas mudanças ainda não foi medida; os pisos do épico seguem
  como metas. Status permanece In Progress até fechar os critérios e a DoD no main.

### Gate local completo, 2026-11-03

`just ci-local` terminou com `Rust gates passed (full)` e `All CI gates passed`.
Validação de tasks, independência, typecheck, testes críticos do frontend, rustfmt, Clippy do
workspace e tipos TypeScript atuais passaram. O grupo novo do core passou com 8 testes;
CLI com 18 (16 novos); desktop com 47 (7 novos). File-opening também passou. Sync backfill,
metrics e realtime passaram respectivamente com 5, 8 e 7 testes. Total novo: 31 testes.
Log local: `/tmp/wingdrive-cov-ci-local.log`.

Esta evidência valida as suites locais e o runner normal. O runner instrumentado completo e o
CI remoto deste branch ainda não foram executados. A medição publicada de referência continua
na COV-01; nenhuma porcentagem nova é atribuída a esta entrega.

### Ampliação das suítes e correções, 2026-11-06

Novas suítes de contrato no core: `addressing_contract_test`, `archive_contract_test`,
`space_contract_test`, `tag_validation_contract_test` e `tag_manager_contract_test`. No desktop:
`files/tests`, `gvfs/tests` e `server/tests` em `apps/tauri/src-tauri/src/`, casos novos em
`launch/tests`, `subscription_test.rs` e o teste isolado do seletor de aplicativos em
`file-opening-linux`. Os bugs que elas acharam e as correções estão em [BUGS.md](BUGS.md).

`scripts/ci-rust.sh` passa a rodar as suítes de contrato puras em todas as plataformas. As que
abrem biblioteca, indexam, arquivam ou movem arquivos rodam só no Linux, pois não houve execução
no Windows. `TestDataDir` agora cria um diretório único por instância e não apaga mais um caminho
fixo, o que permite rodar essas suítes em paralelo.

Validação local: `cargo test --locked -p wingdrive --bin WingDrive` com 82 aprovados e um ignorado
preexistente; `file-opening-linux` com 23; e as suítes de tags e endereçamento do core aprovadas.
Os testes do desktop não abrem janela Tauri, montagem GVFS real nem outra versão do app.

### Cobertura medida após `test/coverage-more`, 2026-11-06

`scripts/coverage-rust.sh` no Linux (Arch, Rust 1.99.0), num worktree sem `node_modules`, com
`WING_SKIP_WEB_BUILD=1`. A primeira execução parou em `wing-task-system --test integration_test`
(teste dependente de ordem, corrigido e registrado em [BUGS.md](BUGS.md)); o passo dos crates
compartilhados rodou de novo e `cargo llvm-cov report` juntou os perfis das duas execuções.
Relatório com `python3 scripts/coverage-report.py <dir>/site <dir>/rust.lcov`.

| Componente | Antes (medição local de 2026-11-06) | Depois |
|---|---|---|
| Core | 30,2% (24.635/81.455) | 44,1% (35.896/81.472) |
| Desktop | 54,7% (3.781/6.912) | 60,8% (4.267/7.014) |

O maior ganho do core vem de rodar todos os testes unitários (`cargo test -p wing-core --lib`,
489 aprovados) no gate Linux, em vez de seis módulos. O denominador do desktop cresceu 102 linhas
com casos novos em módulos `tests` inline; sem elas, a cobertura fica em cerca de 60,3%.

Arquivos alterados, antes e depois: `ops/files/copy/job.rs` 351/655 para 499/663,
`ops/files/copy/strategy.rs` 150/716 para 174/723, `ops/files/delete/strategy.rs` 0/234 para
56/234, `ops/config/app/update.rs` 0/315 para 224/315, `src-tauri/src/main.rs` 235/1455 para
375/1455, `keybinds.rs` 0/46 para 38/46, `file_opening.rs` 0/89 para 55/89, `files.rs` 64/159
para 87/167 e `undo.rs` 143/166 para 201/207.

O teste do desktop que inicia `wing-daemon` usou o binário de outro checkout, ligado em
`target/debug`; ele procura o binário fora de `CARGO_TARGET_DIR`. Windows e macOS não rodaram.
