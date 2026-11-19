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

## Interactive cloud setup regression 2026-11-19

Scripted pseudo-terminal sessions exercise the actual CLI wizard against localhost RPC. They
cover mixed volume listing, removal of the selected cloud volume, decoding a structured removal
response and returning to the menu after adding GCS from a disposable file. An isolated SQLite
RPC test verifies that local rows and credentials are preserved and cloud rows and credentials
are removed. No real provider was contacted. The diagnosis and correction are in [BUGS.md](BUGS.md).

The initial pipe-based fixture hit four request timeouts because the prompts require a terminal.
The pseudo-terminal reuses `portable-pty` already present in the workspace. An intermediate
test build raised E0308 because the registered handler takes `&Arc<Core>`. Wrapping the fixture
core in `Arc` resolved it. The compile step's waiting-for-other-jobs warning accompanied that
same failure. The original logs are under `coverage/goal/cloud-setup-pipe.log`,
`cloud-setup-compile.log`, `cloud-setup-before.log`, `cloud-removal-core-before.log` and
`cloud-removal-core-single-thread-panic.log` in the same directory.

The final RPC suite passed six tests and the adjacent Google, Google Picker and Microsoft
suites passed four. The CLI passed 566 tests including the pseudo-terminal cases. These runs
used LLVM instrumentation on Linux and had zero failures, ignored tests or compiler warnings.
The logs are in `coverage/goal/cloud-removal-final-rpc.log`, `cloud-neighbors.log` and
`cloud-setup-after.log`. The pseudo-terminal suite currently runs on Unix.

## Legacy cloud volume classification

The cloud wizard and removal RPC now use the same classification for provider mounts stored
as `Network`. The fixtures include a legacy S3 mount without cloud fields in the CLI
response and a network share that must stay outside the cloud menu. The SQLite contract
also checks credentials and tracked rows after allowing or refusing each removal.
See [BUGS.md](BUGS.md). Instrumented verification remains pending.

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
