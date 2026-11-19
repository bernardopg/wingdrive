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

The four process cases executed against the previous binary in a standalone fixture. Three passed
and the quoting case reproduced the error. The corrected path helper and matcher passed the
focused harness, and `systemd-analyze verify --man=no --generators=no` accepted the generated
fixture unit. The corrected binary's integrated verification has been queued. Logs are in
`coverage/goal/cli-systemd/{baseline-fixture,helper,systemd-verify}.log`.

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
The instrumented baseline rejected the legacy S3 row with `Volume is not tracked`. The first
PTY regression waited for a cloud listing and timed out when the menu treated the library as
empty. Its fixture now waits for the next menu before checking the rendered provider list.
The corrected combined build passed 584 CLI tests and seven RPC contracts. See [BUGS.md](BUGS.md).
Logs are in `coverage/goal/next/legacy-before-{core,cli}.log` and
`coverage/goal/next/legacy-cloud-and-provider-tests.log` in the primary checkout.

## Dependencies

COV-09, COV-03. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Subir daemon com diretório, instância e endpoints exclusivos de cada teste.
- [ ] Executar library/location/file/search/job/status, verificando JSON, exit code e bytes em disco.
- [ ] Cobrir daemon indisponível, timeout, biblioteca ausente e falha de operação; encerrar apenas PIDs criados pelo teste.

## Acceptance Criteria

- [ ] CLI atinge pelo menos 75% de linhas na medição completa.
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

The corrected CLI passed 581 unit tests and four process tests under LLVM instrumentation.
The combined desktop build reported E0432 because the shell imported the branding crate
directly. The shell now uses the existing dependency on `wing-tauri-core`, which re-exports
the shared helper on Linux. The logs are in `coverage/goal/next/cli.log` and
`coverage/goal/next/desktop.log` in the primary checkout.

An independent coverage replay overlapped a Cargo build and a hard-linked CLI fixture
returned ETXTBSY (OS error 26). Each fixture now copies the executable so that it has its
own inode. The failed replay is recorded in `coverage/goal/next/cli-profile-attempts/service.log` was superseded by an isolated successful
profile replay. The original failing console output is captured in this task.

The instrumented CLI unit binary, process test binary and CLI executables were exported together.
The focused report covered 5,781/7,311 CLI lines (79.1%) using the official filter. The
production-only CLI diagnostic was 3,892/5,421 (71.8%). Both figures come from the isolated
verification tree, pending comparison with a full run on the merged SHA.

### Cloud provider prompt contracts

Three new cases drive the CLI wizard through its existing pseudo-terminal fixture. They
check S3-compatible provider selection, default regions and endpoints, Azure Blob fields
and Dropbox OAuth fields. The fixture sends synthetic values to loopback and never contacts
a provider. All eight cloud wizard tests passed on Linux in the instrumented verification
tree. See `coverage/goal/next/cloud-pty-final.log` in the primary checkout.
