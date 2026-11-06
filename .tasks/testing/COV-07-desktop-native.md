---
id: COV-07
title: "Cobrir shell desktop, IPC e segurança do undo"
status: To Do
assignee: bernardopg
parent: COV-00
priority: High
milestone: M2
tags: [coverage, testing, desktop]
last_updated: 2026-11-06
---

## Description

O shell nativo concentra integrações que testes da interface web não exercitam.

## Dependencies

COV-02. Escopo, metas e ordem em [README.md](README.md).

## Implementation Steps

- [ ] Cobrir launch, single instance, IPC, clipboard, terminal e fila de abertura.
- [ ] Testar undo contra inode substituído, symlink, arquivo alterado e destino existente.
- [ ] Isolar D-Bus, HOME/XDG e processos; registrar casos que exigem runtime gráfico.

## Acceptance Criteria

- [ ] Desktop apps/tauri atinge pelo menos 60% na primeira etapa e 75% na conclusão.
- [ ] Undo nunca sobrescreve dados criados depois da operação original.
- [ ] Testes sem display rodam no CI; evidência Tauri/Linux cobre integrações reais e M3 registra macOS/Windows pendentes.

## Evidence

- 2026-11-06: suítes novas em `apps/tauri/src-tauri/src/{main,keybinds,file_opening,windows}/tests/`
  e casos novos em `launch`, `files`, `gvfs`, `undo`, `terminal`, `background`,
  `filemanager1` e `network_discovery`. Comandos com `tauri::State` rodam num app
  `tauri::test::mock_app`, e o proxy do daemon fala com um listener TCP local de uma conexão.
  `cargo test --locked -p wingdrive --bin WingDrive`: 107 aprovados; o teste que inicia
  `wing-daemon` depende do binário em `<workspace>/target/debug`. Sem janela, D-Bus real ou polkit.
