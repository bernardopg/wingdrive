# Roadmap do WingDrive

Atualizado em 2026-10-28. Método e regras em [PLANNING.md](PLANNING.md).

Foco: WingDrive como **file manager desktop** confiável (UI Tauri + CLI) antes de integrações
entre dispositivos.

## Done — M1: File manager desktop Linux 1.0

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
| 4 | TAURI-006 | Fechar a matriz de regressão Linux (Done) | S02 |
| 5 | BRAND-002 | Wingbot, WingUI, Wingdrop e Wings (Done) | S02 |

Fora da ordem, já Done: FORK-005 (licença Apache-2.0 e port de correções do upstream).
Release `v2.0.0-alpha.7` sai com TAURI-013, TAURI-014 e FORK-005; ver [CHANGELOG](../CHANGELOG.md).

O M1 fecha quando essas cinco estiverem Done e uma release sair sem bug conhecido.

## Now — M2: Gerenciador de arquivos padrão no Linux

Prioridade máxima por decisão do usuário (2026-10-07). Resultado: WingDrive substitui Dolphin,
Nautilus ou Yazi no uso diário. `xdg-open` em pastas e "Mostrar na pasta" de outros apps abrem o
WingDrive, ele fica residente na bandeja e abre na hora, e tem as operações básicas de um file
manager. Navegação continua gráfica; modo modal estilo Yazi fica fora.

Epic: `DESK-000`. Sprints S03 a S05.

| Ordem | Task | Título |
|---|---|---|
| 1 | DESK-001 | Integração XDG, caminhos na linha de comando e instância única |
| 2 | DESK-003 | Segundo plano na bandeja e início com a sessão |
| 3 | DESK-002 | Serviço D-Bus `org.freedesktop.FileManager1` |
| 4 | DESK-004 | "Abrir com" real no Linux |
| 5 | DESK-005 | "Abrir terminal aqui" |
| 6 | DESK-006 | Novo arquivo vazio e link simbólico |
| 7 | DESK-007 | Permissões e dono nas propriedades |
| 8 | DESK-008 | Compactar e extrair |
| 9 | DESK-009 | Rede e dispositivos via GIO/gvfs (MTP, SMB, SFTP) |
| 10 | DESK-010 | Latência de abertura e orçamento de memória |

Pré-requisito: TAURI-015 (pacote AUR com GTK/WebKit do sistema, daemon encerrado ao sair).

Estado em 2026-10-28: S02 e S03 fechados. Done no M2: DESK-001 a DESK-012, DESK-014, DESK-022,
DESK-025 a DESK-028, DESK-036, TAURI-015, INDEX-013, INDEX-014, NET-005, NET-006, SEC-010,
WATCH-007, WATCH-008, FORK-006 e FORK-008. O que falta para fechar o M2 (plano de ataque em
[S04](sprints/S04.md)):

| Task | O que falta | Sprint |
|---|---|---|
| DESK-017 | Um dia inteiro de uso do pacote AUR, sem voltar a outro file manager | S04 |
| DESK-015, DESK-018 a DESK-021, DESK-023, DESK-024 | Código no `main`; falta validar no build Linux (no dia da DESK-017) | S04 |
| DESK-029 a DESK-033, DESK-035 | Código no `main` (#296, #298); falta validar no build Linux; DESK-033 precisa da senha real no polkit | S04 |
| DESK-034 | Terminal embutido (F4) | S04 |
| WATCH-006 | Conexão no login e `wing` no PATH, no pacote seguinte | S04 |
| INDEX-010 | Checagem visual de seleção, tags e troca de biblioteca | S04 |
| FORK-007 | Código no `main` (#231); falta conferir devices pareados com um peer real | S04 |
| DESK-013 | Revisão pt no Crowdin e verificação do pacote (passos 3 a 5) | S04 |
| DESK-016 | Explorer com painel duplo (F3); exige tirar o shell do router de cada aba | S05 |
| FORK-009 | Correções enviadas ao upstream (#3160 a #3162) | S05 |
| DESK-000 | Fecha quando todas as filhas estiverem Done | S05 |

O M2 fecha quando o DESK-000 estiver Done com evidência do pacote AUR e um dia inteiro de uso sem
voltar a outro file manager (DESK-017). Previsão: fim do S05 (2026-11-25).

## Next — M3: Desktop multiplataforma e release

Resultado: mesmo nível do M1 em macOS e Windows, com release público assinado.

- `TAURI-012` matriz de runtime em macOS e Windows (preparação no S05)
- `CORE-014` abrir arquivos e "Abrir com" no macOS e no Windows (roda com a TAURI-012)
- `FORK-002` identidade e release independentes do WingDrive: CI, release e imagem GHCR já rodam no
  repositório; falta a assinatura (plano no S05)
- `FORK-009` correções do WingDrive enviadas ao upstream (#3160 a #3162 em revisão)
- Assinatura e instaladores por plataforma
- `FILE-007` plano de preflight para copiar, mover e apagar (veio do item 7 da FORK-006)
- `FILE-008` journal de jobs e undo para cópia, sobrescrita e lixeira (veio do item 7 da FORK-006)

## Later — M4: Integração entre dispositivos

Pairing, Spacedrop, library sync e file sync confiáveis entre desktops.

- `FSYNC-000` File Sync
- `SHARE-001` arquitetura de compartilhamento
- Revalidar `NET-000` e `LSYNC-000` (marcados Done no upstream) em uso real

## Later — M5: Opcionais

Cloud (`CLOUD-000`, `CLOUD-003`), extensões (`PLUG-000`), Spacebot (`TAURI-004`), IA (`AI-000`),
mobile (`RES-000`, `SEC-009`). Só entram depois de M4, salvo decisão explícita.

Exceção decidida em 2026-10-17: o épico mobile `MOB-000` (Android funcional, loop de dev rápido,
paridade com o desktop) foi puxado para agora. MOB-001 a MOB-005 entram no S03.

## Backlog herdado do upstream

Tasks criadas no Spacedrive (assignee `jamiepine`) que ainda estavam abertas. Na auditoria de
2026-10-16 receberam a tag `upstream-backlog` e uma nota "WingDrive triage" no arquivo, sem apagar
nada:

- M4 (20 tasks): file sync (`FSYNC-*`), compartilhamento público (`SHARE-*`), Spacedrop e Tailscale
  (`NET-003`, `NET-004`), sync de sidecars entre devices (`VSS-004`) e `FILE-006`.
- M5 (45 tasks): IA, cloud, extensões, busca semântica (`SEARCH-*`), criptografia e RBAC (`SEC-*`),
  mobile e volumes virtuais, sidecars, coleções e ingestão, épicos de interface herdados (`UI-000`,
  `UI-001`, `MEDIA-000`, `NAV-000`, `SETS-000`, `SRCH-000`).
- Fechadas como obsoletas: `CORE-012` (registro de tipos em Swift, sem app Swift no repositório) e
  `CORE-013` (substituída pelo `useNormalizedQuery` do `ts-client`).

Listar: `cargo run --bin task-validator -- list --milestone M5 --status "To Do"`.

## Histórico

| Data | Mudança |
|---|---|
| 2026-10-28 | #296 mergeado; S02 e S03 fechados com review e retro; sobras do S03 (DESK-021, DESK-023, DESK-024, WATCH-006, MOB-005, MOB-009) no S04; DESK-034 e as duas falhas da DESK-029 (emblema Git nos filhos e contagem da barra de status) entram no S04; DESK-016 ganha notas de desenho |
| 2026-10-28 | S04: DESK-029 a DESK-033 e DESK-035 com código pronto (PRs #296 e #298), In Progress até a checagem no build Linux; visualização rápida de PDF, texto e código (#295); pdfjs-dist 6.4 fecha o alerta Dependabot 47 (#297); DESK-034 adiada |
| 2026-10-17 | Revisão do app mobile no Moto G7 e no emulador: épico MOB-000 com 11 tasks em `.tasks/mobile/`; MOB-001 a MOB-005 puxadas para o S03 |
| 2026-10-16 | Auditoria do `.tasks/`: quadro do S03 atualizado, S04 e S05 planejados; 65 tasks herdadas do upstream estacionadas em M4/M5; EXPL-000 e SEC-008 Done; CORE-012 e CORE-013 obsoletas |
| 2026-10-15 | M2 quase fechado; candidatas DESK-015, DESK-016 e DESK-017 abertas; app mobile traduzido e com typecheck no CI |
| 2026-10-07 | Novo M2 "Gerenciador de arquivos padrão no Linux" (DESK-000) como prioridade máxima; antigos M2–M4 viram M3–M5 |
| 2026-10-06 | M1 completo: TAURI-006 e BRAND-002 Done; INDEX-010 integrada; sync multi-device corrigido |
| 2026-10-06 | FORK-004, TAURI-014 e FORK-005 Done; licença Apache-2.0; release `v2.0.0-alpha.7` |
| 2026-10-05 | Now atualizado: alpha.6 publicada no AUR; TAURI-013 dividida (TAURI-014); BRAND-002 e TAURI-006 no S02 |
| 2026-10-01 | Roadmap criado; M1 definido a partir da revisão `docs/design/TAURI_FILEMANAGER_REVIEW_2026-09-03.md` |
