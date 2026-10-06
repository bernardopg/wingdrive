# Roadmap do WingDrive

Atualizado em 2026-10-06. Método e regras em [PLANNING.md](PLANNING.md).

Foco: WingDrive como **file manager desktop** confiável (UI Tauri + CLI) antes de integrações
entre dispositivos.

## Now — M1: File manager desktop Linux 1.0

Resultado: no Linux, navegar, abrir, copiar, mover, renomear, criar pasta e apagar arquivos pela
UI e pelo CLI, sem bug conhecido, rodando de um bundle de produção.

Epics: `TAURI-000`, `EXPL-000`, `CLI-000` (estendido por `CLI-001`).

Concluídas no S01: WATCH-003, WATCH-004, CLI-001, EXPL-001/002/003/005/006/007, TAURI-008/009/010/011,
DEV-003, FORK-003, BRAND-001. Release `v2.0.0-alpha.6` publicada no GitHub e no AUR (`wingdrive-bin`).

Restante do M1, em ordem:

| Ordem | Task | Título | Sprint |
|---|---|---|---|
| 1 | TAURI-013 | Estabilização Linux: segurança de dados (Done) | S01 |
| 2 | FORK-004 | Próxima tag atualiza o AUR sem passo manual (Done) | S01 |
| 3 | TAURI-014 | Robustez em runtime e paridade da busca (Done) | S02 |
| 4 | TAURI-006 | Fechar a matriz de regressão Linux | S02 |
| 5 | BRAND-002 | Wingbot, WingUI, Wingdrop e Wings | S02 |

Fora da ordem, já Done: FORK-005 (licença Apache-2.0 e port de correções do upstream).
Release `v2.0.0-alpha.7` sai com TAURI-013, TAURI-014 e FORK-005; ver [CHANGELOG](../CHANGELOG.md).

O M1 fecha quando essas cinco estiverem Done e uma release sair sem bug conhecido.

## Next — M2: Desktop multiplataforma e release

Resultado: mesmo nível do M1 em macOS e Windows, com release público assinado.

- `TAURI-012` matriz de runtime em macOS e Windows
- `FORK-002` identidade e release independentes do WingDrive
- Assinatura e instaladores por plataforma

## Later — M3: Integração entre dispositivos

Pairing, Spacedrop, library sync e file sync confiáveis entre desktops.

- `FSYNC-000` File Sync
- `SHARE-001` arquitetura de compartilhamento
- Revalidar `NET-000` e `LSYNC-000` (marcados Done no upstream) em uso real

## Later — M4: Opcionais

Cloud (`CLOUD-000`, `CLOUD-003`), extensões (`PLUG-000`), Spacebot (`TAURI-004`), IA (`AI-000`),
mobile (`RES-000`). Só entram depois de M3, salvo decisão explícita.

## Histórico

| Data | Mudança |
|---|---|
| 2026-10-06 | FORK-004, TAURI-014 e FORK-005 Done; licença Apache-2.0; release `v2.0.0-alpha.7` |
| 2026-10-05 | Now atualizado: alpha.6 publicada no AUR; TAURI-013 dividida (TAURI-014); BRAND-002 e TAURI-006 no S02 |
| 2026-10-01 | Roadmap criado; M1 definido a partir da revisão `docs/design/TAURI_FILEMANAGER_REVIEW_2026-09-03.md` |
