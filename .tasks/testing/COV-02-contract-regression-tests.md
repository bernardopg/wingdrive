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
