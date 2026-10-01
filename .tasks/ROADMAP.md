# Roadmap do WingDrive

Atualizado em 2026-10-01. Método e regras em [PLANNING.md](PLANNING.md).

Foco: WingDrive como **file manager desktop** confiável (UI Tauri + CLI) antes de integrações
entre dispositivos.

## Now — M1: File manager desktop Linux 1.0

Resultado: no Linux, navegar, abrir, copiar, mover, renomear, criar pasta e apagar arquivos pela
UI e pelo CLI, sem bug conhecido, rodando de um bundle de produção.

Epics: `TAURI-000`, `EXPL-000`, `CLI-000` (estendido por `CLI-001`).

| Ordem | Task | Título | Sprint |
|---|---|---|---|
| 1 | TAURI-006 | Matriz de regressão em runtime (Linux) | S01 |
| 2 | CLI-001 | `sd-cli file rename/delete/mkdir` | S01 |
| 3 | EXPL-006 | Direção de ordenação (asc/desc) | S01 |
| 4 | TAURI-008 | Atalhos padrão: refresh, nova pasta, ocultos | S01 |
| 5 | TAURI-009 | Trocar `alert()` nativo por toast | S01 |
| 6 | TAURI-010 | Clipboard integrado ao sistema | S02 |
| 7 | EXPL-007 | Abrir symlinks corretamente | S02 |
| 8 | TAURI-011 | Bundle de produção Linux verificado | S02 |
| 9 | EXPL-005 | Abas com estado isolado | S03 |

Backlog M1 sem sprint: `EXPL-001`, `EXPL-002`, `EXPL-003` (critérios herdados do upstream,
revisar e fechar ou quebrar em tasks novas) e bugs que o `TAURI-006` encontrar.

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
