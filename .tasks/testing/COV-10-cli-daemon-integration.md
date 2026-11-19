---
id: COV-10
title: "Testar CLI com daemon isolado e filesystem real"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M3
sprint: S06
tags: [coverage, testing, cli]
last_updated: 2026-11-03
---

## Description

Testes de parsing não provam que wing envia requests corretos, aguarda jobs e informa falhas com exit code adequado.

## Linux daemon service cycle 2026-11-19

The CLI emitted an unquoted `ExecStart` for paths containing spaces and special characters
when run with a fake `systemctl`. The fixture uses a
disposable HOME, a temporary executable directory, a stub daemon and a command log. The
diagnosis and correction are recorded in [BUGS.md](BUGS.md). The baseline output is in
`coverage/goal/cli-systemd/before.log` from the primary checkout. A direct Rust harness using cached dependencies failed with
E0463 loading `tempfile`. This isolated linker failure is in
`coverage/goal/cli-systemd/rustc-harness.log`; use the Cargo integration binary for verification.

## Dependencies

COV-09, COV-03. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Subir daemon com diretório, instância e endpoints exclusivos de cada teste.
- [ ] Executar library/location/file/search/job/status, verificando JSON, exit code e bytes em disco.
- [ ] Cobrir daemon indisponível, timeout, biblioteca ausente e falha de operação; encerrar apenas PIDs criados pelo teste.

## Acceptance Criteria

- [ ] CLI atinge pelo menos 60% de linhas na medição completa.
- [ ] Cópia/move/rename/delete/mkdir têm cenários de sucesso e erro no CLI real.
- [ ] Sem tocar daemon, serviços, portas ou bibliotecas do autor; teardown funciona após falhas.

## Loopback RPC evidence 2026-11-19

Thirteen tests exercise account listing and removal, Google and Microsoft OAuth client settings,
sign-in polling, Drive and OneDrive mount dispatch, Photos albums and picker import, and cloud sync
pair creation, presentation, execution, deletion and invalid URI handling. The fixture starts a
loopback TCP server and asserts the method, library and payload. It uses synthetic accounts,
disabled browser launch and disposable local paths.

The Linux CLI binary passed 574 tests with LLVM instrumentation. Separate profiles from that exact
binary, with and without the 13 new tests, show `domains/integrations/mod.rs` improving from 0/400
to 369/400 lines. Both profile runs had zero failures or ignored tests, and the focused build
emitted no compiler warnings. Logs and LCOV files are under `coverage/goal/cli-integration-rpc/`.
This measurement uses only the CLI binary. The full component measurement and the process-level
daemon cases remain open.
