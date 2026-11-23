---
id: COV-07
title: "Cobrir shell desktop, IPC e segurança do undo"
status: In Progress
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
sprint: S05
tags: [coverage, testing, desktop]
last_updated: 2026-11-23
---

## Description

O shell nativo concentra integrações que testes da interface web não exercitam.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [x] Cobrir launch, single instance, IPC, clipboard, terminal e fila de abertura.
- [x] Testar undo contra inode substituído, symlink, arquivo alterado e destino existente.
- [x] Isolar D-Bus, HOME/XDG e processos; registrar casos que exigem runtime gráfico.

## Acceptance Criteria

- [x] Desktop apps/tauri atingiu as metas intermediárias de 60% e 75%.
- [ ] Desktop apps/tauri atinge 100% de linhas instrumentadas na medição completa.
- [x] Undo nunca sobrescreve dados criados depois da operação original.
- [x] Testes sem display rodam no CI; evidência Tauri/Linux cobre integrações reais e M3 registra macOS/Windows pendentes.

## Reabertura da meta de cobertura

O trabalho de 13 de novembro cumpriu a meta anterior do shell desktop. A meta vigente de 100%
continua aberta. O relatório Linux tem 8.035/8.910 linhas cobertas (90,2%). A validação COV-12
registra o SHA, a coleta e as 875 linhas que faltam na medição.

## GTK clipboard coverage 2026-11-19

The Linux coverage collector now explicitly runs the existing GTK clipboard integration test.
It creates a private Xvfb display and checks GNOME, KDE and plain URI clipboard targets.
The coverage workflow installs Xvfb if the runner does not already have it.
Both instrumented runs passed with zero failures or compiler warnings. Adding the clipboard
profile to the existing partial baseline of the same `clipboard.rs` source covers 88/95 lines
instead of 45/95. The focused run and LCOV are under `coverage/goal/desktop-clipboard/`.
The full desktop measurement is tracked separately.

## Desktop updater checks 2026-11-19

Four updater tests passed with LLVM on Linux. The new tests serve a synthetic release
manifest, an empty response and malformed JSON on loopback. They exercise the plugin
through the desktop lookup function without downloading an installer. The log is
`coverage/goal/next/updater.log` in the primary checkout.

## Evidence

- 2026-11-06: suítes novas em `apps/tauri/src-tauri/src/{main,keybinds,file_opening,windows}/tests/`
  e casos novos em `launch`, `files`, `gvfs`, `undo`, `terminal`, `background`,
  `filemanager1` e `network_discovery`. Comandos com `tauri::State` rodam num app
  `tauri::test::mock_app`, e o proxy do daemon fala com um listener TCP local de uma conexão.
  `cargo test --locked -p wingdrive --bin WingDrive`: 107 aprovados; o teste que inicia
  `wing-daemon` depende do binário em `<workspace>/target/debug`. Sem janela, D-Bus real ou polkit.

### 2026-11-10

Linux x86_64, rustc 1.99.0. Referência local `f9875bfa8`; alterações em
`eaa15d0e3` e `832cb179f`. Foram acrescentados 36 testes unitários: 28 no shell,
6 em `file-opening-linux` e 2 em `file-clipboard-windows`.

Módulos ampliados: `undo` (oito casos), `launch` e fila `PendingOpenRequests`,
pool de IPC em `main`, `clipboard`, `terminal`, `server`, `gvfs`,
`network_discovery`, `filemanager1` e `background`. Nos helpers, foram cobertos
`desktop_entry`, lançamento de terminal e decodificação de `DROPFILES`.
Os casos incluem links quebrados, substituição por symlink, diretório substituído,
destino ocupado, alteração de mesmo tamanho, Range malformado e limites de u64,
erros de persistência e argumentos com metacaracteres.

| Suíte | Aprovados antes | Aprovados depois | Falhas antes/depois | Ignorados antes/depois |
|---|---|---|---|---|
| `wingdrive --bin WingDrive` | 110 | 138 | 1 / 1 | 1 / 1 |
| `file-opening-linux`, unitários | 22 | 28 | 0 / 0 | 0 / 0 |
| `file-opening-linux`, doctests | 1 | 1 | 0 / 0 | 0 / 0 |
| `file-clipboard-windows`, unitários | 5 | 7 | 0 / 0 | 0 / 0 |

Comandos da referência, antes das alterações:

```bash
CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p wingdrive --bin WingDrive
CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p file-opening-linux
CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p file-clipboard-windows
```

Comandos finais, executados um por vez, sem display nem endereço de sessão D-Bus:

```bash
CARGO_TARGET_DIR=$PWD/target env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS cargo test --locked -j 6 -p wingdrive --bin WingDrive
CARGO_TARGET_DIR=$PWD/target env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS cargo test --locked -j 6 -p file-opening-linux
CARGO_TARGET_DIR=$PWD/target env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS cargo test --locked -j 6 -p file-clipboard-windows
CARGO_TARGET_DIR=$PWD/target cargo fmt --all -- --check
CARGO_TARGET_DIR=$PWD/target cargo clippy --locked -j 6 -p wingdrive --all-targets -- -D warnings
CARGO_TARGET_DIR=$PWD/target cargo clippy --locked -j 6 -p file-opening-linux -p file-clipboard-windows --all-targets -- -D warnings
```

Formatação e ambos os comandos Clippy passaram. Na última execução, o shell levou
15,07 s para compilar e 1,15 s para executar; o helper Linux levou 1,53 s e 0,08 s
(unitários e doctest), e o clipboard levou 0,80 s e menos de 0,01 s. A compilação
inicial do shell, com target vazio, levou 15 min 10 s.

A única falha do shell, antes e depois, foi
`start_daemon_integration_tests::named_instance_spawns_isolated_reachable_daemon`:
o build do Tauri deixou `target/debug/wing-daemon` como placeholder vazio, sem
permissão de execução, e o spawn retornou `PermissionDenied`. Não foi compilado
o daemon. O teste de polkit já estava ignorado e permaneceu assim.

O teste de autostart agora chama o comando original num subprocesso com HOME/XDG
temporários, sem alterar o ambiente compartilhado do runner. Os fixtures novos
usam temporários e não usam rede, D-Bus, clipboard real ou configurações do usuário.
O caso de quoting executa apenas `/bin/sh` com HOME temporário no subprocesso.
A suíte já existente mantém seus listeners TCP locais e testes de PTY.

O teste `escaped_percent_file_codes_remain_literal_and_still_append_files`
reproduziu a perda dos arquivos selecionados antes da correção: 0 aprovados,
1 falha, 27 filtrados com
`CARGO_TARGET_DIR=$PWD/target cargo test --locked -j 6 -p file-opening-linux escaped_percent_file_codes_remain_literal_and_still_append_files`.
O bug de expansão do Exec e o isolamento do autostart estão em [BUGS.md](BUGS.md).

Limites: sem integração gráfica, D-Bus real, polkit, daemon real, API Win32 ou
runtime macOS/Windows. Não foi simulada troca concorrente de inode entre a
checagem e a operação no sistema de arquivos. Não foi gerado LCOV nem relatório
de linhas nesta entrega; as metas de 60%/75% continuam sem comprovação atual.
A task passou a In Progress com o merge do #359; os critérios seguem abertos até a medição LCOV.

### 2026-11-11

Linux x86_64, rustc 1.99.0, `cargo-llvm-cov 0.9.1`, branch `test/desktop-coverage` a partir
de `27e8546fe`. Medição com o mesmo filtro de `scripts/coverage-report.py` (componente
`apps/tauri/`, sem arquivos sob `tests/`), depois de `cargo llvm-cov clean --workspace`:

| Medida | Linhas cobertas/instrumentadas | Percentual |
|---|---:|---:|
| `main` (`summary.json` do branch `badges`, SHA `27e8546fe`) | 4.694/7.442 | 63,1% |
| Este branch | 6.352/7.793 | 81,5% |

```bash
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report --locked -p wingdrive --bin WingDrive
cargo llvm-cov --no-report --locked -p file-opening-linux -p file-opening -p file-clipboard-windows -p wing-tauri-core
cargo llvm-cov report --lcov --output-path target/cov-desktop/desktop.lcov
```

Suítes: `wingdrive --bin WingDrive` 184 aprovados, 0 falhas, 1 ignorado (polkit);
`file-opening-linux` 31 unitários e 1 doctest. Os testes novos usam `tauri::test::mock_app`,
listeners TCP no lugar do daemon, scripts `sh` no lugar de `wing-daemon`, `gio`,
`pkexec` e `avahi-browse`, e uma conexão D-Bus ponto a ponto (`zbus` com `p2p` só nos
testes) no lugar do servidor de notificações. Nada usa o daemon, o D-Bus de sessão ou as
configurações do usuário, exceto leituras: tema de ícones, service menus e o shell de login
no teste de terminal, que só roda no Unix.

Para rodar no `MockRuntime`, comandos e helpers passaram a ser genéricos em `tauri::Runtime`
(janelas, menu, assinaturas, bandeja, `launch`, notificações, terminal). Partes que chamavam
processos ou relógios fixos ganharam uma variante que recebe o comando ou o limite de tempo:
`spawn_daemon`, `find_daemon_binary_in`, `wait_for_daemon_within`, `run_mount_command`,
`run_elevated`, `browse_with` e `current_theme_from`.

Novos casos cobrem: menu nativo e seus eventos, abertura de biblioteca, seleção e validação
da biblioteca, assinaturas de eventos (canal por janela, fim do stream, cancelamento, pool
persistente), ciclo de vida do daemon (achar, iniciar, esperar, SIGTERM e kill), todas as
janelas e o posicionamento de menu e overlay, rótulos da bandeja, montagem gvfs, retry com
polkit, FileManager1, descoberta mDNS, terminal real, tema de ícones, service menus e
notificações com cliques.

Os testes acharam seis bugs, corrigidos e descritos em [BUGS.md](BUGS.md) (entradas
"Desktop:"): "Open Library..." tratava a recusa do daemon como sucesso, o ID da biblioteca
entrava sem escape no script das janelas, a assinatura padrão não recebia o Wingdrop, rotas
de janela não codificavam IDs, o overlay de voz ficava escondido sem monitor e a descoberta
mDNS perdia servidores acima de 64 KiB de listagem.

Fora da medição por precisarem de runtime gráfico, GTK ou macOS: `main()`, a criação da
bandeja, o clipboard GTK, `drag/` e `file_drag.rs` (macOS), o diálogo de "Open Library..." e
a instalação do serviço do daemon, que escreve em `~/.config/systemd`.

### 2026-11-12

Linux x86_64, base `6152c8ecd`. A entrega parcial acrescenta testes da fila e do
encaminhamento de argumentos em um D-Bus privado, dos comandos e do host do seletor
de arquivos com runtime Tauri simulado, e da reversão de arquivos substituídos,
links e diretório pai substituído no journal do core. O runtime simulado não
confirma a destruição real da janela. Os testes anteriores já cobrem mudança de
mesmo tamanho e destino ocupado. A troca concorrente entre verificar o selo e
alterar o arquivo não foi testada; o critério de segurança permanece aberto.

| Componente ou arquivo | Antes | Depois |
|---|---:|---:|
| Desktop (`apps/tauri/`) | 6.738/8.371 (80,5%) | 6.823/8.379 (81,4%) |
| `launch.rs` | 172/201 (85,6%) | 197/209 (94,3%) |
| `file_chooser.rs` | 85/191 (44,5%) | 133/191 (69,6%) |

Medição com `cargo llvm-cov --no-report` para `wingdrive --bin WingDrive` e
`file-opening-linux`, `file-opening`, `file-clipboard-windows` e `wing-tauri-core`,
seguido de `cargo llvm-cov report --lcov` e `scripts/coverage-report.py`. Perfis
foram limpos antes da medição final. A meta desta rodada, 88%, permanece aberta.
Com display e endereço D-Bus de sessão ausentes, o shell teve 214 aprovados e um
ignorado (`pkexec_round_trip_on_root_owned_files`, exige agente polkit). Os helpers
tiveram 31, 1, 7 e 0 testes aprovados, respectivamente. O core, com HOME/XDG
temporários, teve 609 testes unitários aprovados e dois ignorados.

`cargo fmt --all`, Clippy do shell com `--all-targets -D warnings` e Clippy da
biblioteca do core passaram. Clippy do core com `--all-targets -D warnings` falhou
em avisos de testes não alterados, incluindo `archive_contract_test.rs` e helpers
compartilhados. A integração `file_undo_test` teve 18 aprovados com HOME/XDG pelo
caminho físico, mas 17 aprovados e uma falha com HOME pelo symlink `target`; a
mesma falha foi reproduzida na base limpa `6152c8ecd`. O caso está em [BUGS.md](BUGS.md).
Não houve validação gráfica ou de polkit real. A cobertura e os
critérios restantes mantêm a tarefa em andamento.

## Regressão do subprocesso GIO

O [registro em BUGS.md](BUGS.md#montagem-gvfs-pode-expirar-antes-de-ler-o-diagnóstico-do-gio)
detalha a espera circular observada quando o GIO escreve mais dados do que o pipe comporta.

A correção usa I/O assíncrono e mantém o prazo da operação enquanto lê stderr.
`cargo test --locked --offline -p wing-core -p wing-cli -p wing-bench -p wingdrive --bin WingDrive gvfs::`
com ambiente LLVM passou 11 testes em Linux. Os primeiros harnesses avulsos falharam
com E0463 ao resolver dependências de Tokio. A suíte do binário recompilou o código com
o grafo correto. A medição focada ainda deixa a meta global de desktop em aberto.

### 2026-11-13

Cobertura de linhas do componente desktop (`apps/tauri/`, sem arquivos de teste): **80,5% para 91,6%**
(7.985 de 8.718), medida com `cargo llvm-cov` sobre `wingdrive --bin WingDrive`, `wing-tauri-core`,
`file-opening`, `file-opening-linux` e `file-clipboard-windows`. `cargo test -p wingdrive --bin WingDrive`:
223 aprovados, 1 ignorado. Clippy `-D warnings` limpo no desktop e no core com `--all-targets`.

| Arquivo | Antes | Depois |
|---|---|---|
| `main.rs` | 67,2% | 85,3% |
| `drag/mod.rs`, `drag/session.rs` | 0% | 99,0%, 100% |
| `drag/commands.rs` | 0% | 88,9% |
| `files.rs` | 72,3% | 94,2% |
| `file_opening.rs` | 61,8% | 92,1% |
| `notifications.rs` | 86,2% | 97,2% |
| `background.rs` | 80,7% | 96,0% |
| `server.rs`, `windows.rs`, `admin.rs` | 85-90% | 95,8%, 95,3%, 96,6% |
| `network_discovery.rs` | 91,4% | 97,5% |
| `file_chooser/portals_conf.rs`, `file_chooser.rs` | 86,2%, 69,6% | cobertos `from_vars`, troca do switch e reinício do portal |

Bugs corrigidos (detalhes em [BUGS.md](BUGS.md)): undo da lixeira com pasta-pai por symlink,
sessão de arraste presa no Linux e no Windows, frontend sem forma de encerrar um arraste, unit systemd
do daemon quebrando caminhos com espaço, e import de exports de location anteriores ao fork
(o rebrand comparava o mesmo cabeçalho duas vezes).

Ficam sem cobertura por exigirem display ou sessão D-Bus real: `clipboard.rs` e `file_drag.rs` (GTK),
`serve` de `filemanager1.rs` e `file_chooser.rs`, e partes de `gvfs.rs` que chamam o `gio` do sistema.

### 2026-11-19

Seven HTTP server tests passed under LLVM instrumentation on Linux. Three new tests exercise
full and partial media responses, suffix and open-ended ranges, unsatisfiable and malformed ranges,
zero-byte files, missing paths, and sidecar resolution, MIME types and authorization. The fixtures
use a temporary directory and a loopback port.

Profiles of the same test binary show 209/438 production lines of `server.rs` covered with its
existing tests and 368/438 with all seven. There were zero failures, ignored tests or compiler
warnings. Logs and LCOV are under `coverage/goal/desktop-server/`. The full desktop report is pending.

### GVFS device-list failure and retry

The Linux command test reproduced a successful empty result for a GIO process that exited 1 with
a diagnostic. The regression now verifies successful empty output, stderr on failure, exit status
without stderr and failure to start the executable. All 12 instrumented GVFS tests passed.
The sidebar test verifies the error indicator and a successful retry through React Query. It passed
with five assertions and runs in CI and the frontend coverage collector.

See [BUGS.md](BUGS.md#desktop-a-listagem-de-volumes-do-gio-escondia-falhas) for the cause and fix.
The logs are in `coverage/goal/next/gvfs-list-*.log` and `gvfs-network-ui.log`. Fixtures used
local subprocesses and a simulated device. System GIO and real phone connections were outside
this verification.

### FileManager1 on a private session bus

The FileManager1 contract test starts its own D-Bus daemon and verifies the requests delivered
by ShowFolders, ShowItems and ShowItemProperties for a temporary directory and a file with a
space in its name. It shares the existing private bus fixture with the file chooser.
The combined Linux desktop suite passed 319 tests, including both D-Bus integrations, with
two existing ignores. Separate LLVM profiles of the same binary put instrumented coverage of
`filemanager1.rs` at 102/163 before the new case and 134/163 with it. The denominator includes
existing inline tests. The full component target stays open. Logs and profiles are in
`coverage/goal/next/` in the primary checkout.

### Linux terminal discovery

The file opener now has a subprocess test for Linux terminal discovery with a private PATH.
It verifies the priority of `xdg-terminal-exec`, a configured `TERMINAL`, known fallback
emulators, and the absence of a terminal. All five terminal tests passed under LLVM
instrumentation. The test checks the selected executable and argument boundaries.
### Linux resident launch forwarding

The private D-Bus test starts a child process and exercises the path that forwards its arguments
and working directory to a running instance. It also checks the result when the destination is
absent. A separate mock-app case verifies that a hidden autostart queues nothing and that a
hidden invocation with a folder delivers the request. All 21 launch tests passed with LLVM
instrumentation. The focused profile hit 219/227 lines in `launch.rs`, including 16 lines missed
by the previous full collector. This test uses a private bus and the mock window runtime.

### Linux default opener

A child process with a private PATH verifies that the desktop's default opener falls back
from a failing `xdg-open` to GIO and passes a directory containing spaces as one argument.
With both launchers unavailable or failing, the command returns a platform error. All five
instrumented file-opening tests passed. See `coverage/goal/after-sidecars/default-opener.log`.

### Linux native startup and menu

The installed binary and the instrumented checkout both opened a visible window, connected to
an isolated daemon and exited after an X11 close event. The instrumented baseline emitted eleven
GTK accelerator warnings. The corrected binary passed the same probe with zero accelerator
warnings. All eight menu tests also passed. See [BUGS.md](BUGS.md).

The private X11, D-Bus and network namespace logged document-portal FUSE limitations and
multicast `ENODEV`. The frontend build reported eight MapLibre BigInt target warnings and one
ineffective Tauri event import warning. The build and the startup probe both returned zero.
Logs are under `coverage/goal/after-watchers/native-smoke/`. The runtime profile remains
separate from the last complete collector.

## Medição completa em 2026-11-20

| Execução Linux x86_64 | Linhas cobertas | Cobertura | Linhas sem execução |
|---|---:|---:|---:|
| [CI da main em `1337cc0df`](https://github.com/bernardopg/wingdrive/actions/runs/44630889175) | 8.069 / 8.958 | 90,1% | 889 |
| Coleta oficial em `e9d2388a4cd0bfb1dea19b1977dfef42c81b5499` | 8.244 / 8.958 | 92,0% | 714 |
| Mesma coleta com perfil do binário nativo | 8.388 / 8.958 | 93,6% | 570 |

Os coletores oficiais passaram com Rust 1.100.0, cargo-llvm-cov 0.9.1 e Bun 1.4.5.
Foram 2.234 testes Rust e 371 de frontend sem falhas, com quatro casos Rust ignorados.
O teste GTK de clipboard passou em Xvfb. A coleta preservou os filtros do repositório.
Os subprocessos verificam as montagens GIO, a troca e restauração do seletor de arquivos,
a remoção do serviço systemd e os comandos de atualização. O atualizador emite progresso
para o download de teste e preserva o AppImage quando a assinatura é inválida.

O perfil suplementar iniciou e encerrou a janela nativa, verificou o ping do daemon e
o encaminhamento da segunda instância, chamou FileManager1 e confirmou que fechar a janela
do seletor cancela o pedido D-Bus. A execução usou Xvfb, D-Bus, HOME e rede privados.
As maiores lacunas após esse perfil estão em `main.rs` (182 linhas), `server.rs` e
`file_drag.rs` (34 cada) e `updates.rs` (29). Faltam, entre outros fluxos, o arraste GTK
entre apps e a instalação seguida de reinício do atualizador. macOS e Windows não entram
nesta medição Linux.

Os relatórios ficam em `coverage/cov07/site/` e `coverage/cov07/with-native/` no worktree.
O Cargo registrou dois avisos de limpeza do cache copiado por ausência de `CACHEDIR.TAG`.
O marcador foi restaurado. A coleta usou perfis novos e terminou com código zero.
O frontend emitiu dois erros esperados no teste de renomeação. O build da interface
indicou oito avisos de BigInt do MapLibre e um de importação dinâmica do evento Tauri.
No smoke, a inicialização recusou duas conexões antes de o daemon responder. Os avisos
de portal, PipeWire, AT-SPI, DRI3, AppIndicator e multicast constam do log em
`coverage/cov07/native/instrumented.log`. O smoke terminou com código zero.

### 2026-11-22

Avanço focado em lacunas testáveis do shell desktop sem GUI real, no worktree
`/home/bitter/.cache/opencode/wt/cov-07` (branch `test/cov-07-desktop-native`).

- `apps/tauri/src-tauri/src/server/tests/mod.rs`: novos testes unitários e de
  handler para caminhos puros e rejeições do servidor HTTP local:
  `parse_range` (formas válidas, sufixo, aberto e rejeições ambíguas),
  `content_type_for` (mapeamentos conhecidos/desconhecidos), `thumb_bucket`
  (clamping), `thumb_kind` (imagem/vídeo/PDF/desconhecido) e `serve_thumb`
  (token inválido, caminho relativo, mídia não suportada, diretório, diretório
  com extensão de imagem e arquivo ausente).
- `apps/tauri/src-tauri/src/updates/tests/http.rs`: teste de app mockado para
  `updater_info`, validando versão atual e canal reportado.

Validação direcionada (Linux x86_64, Rust 1.100.0, `CARGO_TARGET_DIR=$PWD/target`):

```bash
cargo fmt -p wingdrive -- --check
cargo test --locked -j 4 -p wingdrive --bin WingDrive server::contract_tests
cargo test --locked -j 4 -p wingdrive --bin WingDrive server::tests
cargo test --locked -j 4 -p wingdrive --bin WingDrive updates::tests
```

Resultados: fmt OK; `server::contract_tests` 24 passed, 0 failed; `server::tests`
7 passed, 0 failed; `updates::tests` 6 passed, 0 failed. Sem execução de suíte
GUI/Wayland/GTK real, sem polkit real e sem medição LCOV completa nesta rodada.
A meta de 100% instrumentada em medição completa permanece aberta, com lacunas
conhecidas em áreas dependentes de runtime gráfico/native (ex.: `main.rs`,
`file_drag.rs`, `server.rs`, `updates.rs` conforme medição 2026-11-20).

## Realignment (2026-11-23)

- PR #503 merged with server/updater desktop tests. No real GUI/polkit/LCOV full run; 100% desktop lines target unmet. Remains In Progress.
