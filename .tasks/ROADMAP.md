# Roadmap do WingDrive

Atualizado em 2026-10-02. Método e regras em [PLANNING.md](PLANNING.md).

Foco: WingDrive como **file manager desktop** confiável (UI Tauri + CLI) antes de integrações
entre dispositivos.

## Now — M1: File manager desktop Linux 1.0

Resultado: no Linux, navegar, abrir, copiar, mover, renomear, criar pasta e apagar arquivos pela
UI e pelo CLI, sem bug conhecido, rodando de um bundle de produção.

Epics: `TAURI-000`, `EXPL-000`, `CLI-000` (estendido por `CLI-001`).

| Ordem | Task | Título | Sprint |
|---|---|---|---|
| 0 | WATCH-003 | Watcher sincronizar criação/remoção (P0) | S01 |
| 1 | TAURI-006 | Matriz de regressão em runtime (Linux) | S01 |
| 2 | CLI-001 | `wing-cli file rename/delete/mkdir` | S01 |
| 3 | EXPL-006 | Direção de ordenação (asc/desc) | S01 |
| 4 | TAURI-008 | Atalhos padrão: refresh, nova pasta, ocultos | S01 |
| 5 | TAURI-009 | Trocar `alert()` nativo por toast | S01 |
| 6 | TAURI-010 | Clipboard integrado ao sistema | S01 |
| 7 | EXPL-007 | Abrir symlinks corretamente | S01 |
| 8 | TAURI-011 | Bundle de produção Linux verificado | S01 |
| 9 | EXPL-005 | Abas com estado isolado | S01 |

O usuário incluiu EXPL-001, EXPL-002, EXPL-003 e BRAND-001 no S01; implementação e validação concluídas. EXPL-005 e TAURI-011 foram verificadas em produção. FORK-004 segue na publicação GitHub e AUR. TAURI-006 está adiada por decisão explícita do usuário.

Backlog M1: concluir a matriz TAURI-006 e os bugs que ela encontrar. DEV-003 e WATCH-004 já estão Done.

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
| 2026-10-01 | Roadmap criado; M1 definido a partir da revisão `docs/design/TAURI_FILEMANAGER_REVIEW_2026-09-03.md` |
