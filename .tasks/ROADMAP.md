# Roadmap do WingDrive

Atualizado em 2026-11-13. Método e regras em [PLANNING.md](PLANNING.md).

Foco: WingDrive como **file manager desktop** confiável (UI Tauri + CLI) antes de integrações
entre dispositivos.

## Calendário de sprints

Desde 2026-11-13 toda task aberta tem sprint e toda task tem milestone e responsável (`bernardopg`).
Os sprints futuros são um plano de backlog: ao abrir cada um, revisar a lista com o que sobrou.

| Sprint | Datas | Milestone | Foco |
|---|---|---|---|
| [S05](sprints/S05.md) | 2026-11-12 a 2026-11-25 | M2, M3 | Fechar o M2 (DESK-000, BRAND-003 Wings, cobertura) |
| [S06](sprints/S06.md) | 2026-11-26 a 2026-12-09 | M3, M5 | Release assinado, macOS/Windows, Google para qualquer usuário |
| [S07](sprints/S07.md) | 2026-12-10 a 2026-12-23 | M4 | Sync de arquivos e compartilhamento |
| [S08](sprints/S08.md) | 2026-12-24 a 2027-01-06 | M4 | Fechar o M4 |
| [S09](sprints/S09.md) a [S14](sprints/S14.md) | 2027-01-07 a 2027-03-31 | M5 | Opcionais por prioridade |

## Done — M0: Base herdada do upstream

Tasks concluídas no Spacedrive antes do fork, sem milestone até 2026-11-13. Ficam como registro do
que o WingDrive herdou pronto; nenhuma é revalidada aqui (o M4 revalida `NET-000` e `LSYNC-000`).
Listar: `cargo run --bin task-validator -- list --milestone M0`.

## Done — M1: File manager desktop Linux 1.0

Resultado: no Linux, navegar, abrir, copiar, mover, renomear, criar pasta e apagar arquivos pela
UI e pelo CLI, sem bug conhecido, rodando de um bundle de produção.

Epics: `TAURI-000`, `EXPL-000`, `CLI-000` (estendido por `CLI-001`).

Concluídas no S01: WATCH-003, WATCH-004, CLI-001, EXPL-001/002/003/005/006/007, TAURI-008/009/010/011,
DEV-003, BRAND-001 e a renomeação para `wing`. Release `v2.0.0-alpha.6` publicada no GitHub e no AUR (`wingdrive-bin`).

Restante do M1, em ordem:

| Ordem | Task | Título | Sprint |
|---|---|---|---|
| 1 | TAURI-013 | Estabilização Linux: segurança de dados (Done) | S01 |
| 2 | — | Próxima tag atualiza o AUR sem passo manual (Done) | S01 |
| 3 | TAURI-014 | Robustez em runtime e paridade da busca (Done) | S02 |
| 4 | TAURI-006 | Fechar a matriz de regressão Linux (Done) | S02 |
| 5 | BRAND-002 | Wingbot, WingUI, Wingdrop e Wings (Done) | S02 |

Fora da ordem, já Done: licença Apache-2.0 e port de correções do upstream (#114, #115).
Release `v2.0.0-alpha.7` sai com TAURI-013, TAURI-014 e essas correções; ver [CHANGELOG](../CHANGELOG.md).

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

Estado em 2026-11-02: S01, S02 e S03 fechados. Done no M2: DESK-001 a DESK-012, DESK-014,
DESK-015, DESK-018 a DESK-032, DESK-034 a DESK-040, TAURI-015, INDEX-013, INDEX-014,
NET-005 a NET-007, SEC-010, WATCH-007, WATCH-008 e os portes do upstream (#230, #234, #235). A checagem
no pacote de 2026-11-02 fechou 16 tasks de uma vez (log do [S04](sprints/S04.md)). O que falta para
fechar o M2:

| Task | O que falta | Sprint |
|---|---|---|
| DESK-043 | O renderer ainda abre em dois modos: dez rodadas da `v3.0.0` ficaram estáveis, mas só quatro abaixo de 550 MiB. Falta achar os 65 MiB do modo alto | S04 |
| DESK-000 | Fecha quando todas as filhas estiverem Done | S05 |

INDEX-010, DESK-041 e DESK-042 fecharam em 2026-11-03. DESK-017 (vários dias de uso do pacote AUR,
relato do autor) e DESK-033 (`pkexec` e polkit reais na máquina do autor) fecharam em 2026-11-05, junto com
WATCH-006 (conexão no login confirmada no journal), DESK-016 (painel duplo no pacote) e DESK-013
(revisão completa do pt-BR no Crowdin e AppImage conferido no Hyprland).

O M2 fecha quando o DESK-000 estiver Done com evidência do pacote AUR e um dia inteiro de uso sem
voltar a outro file manager (DESK-017). Previsão: fim do S05 (2026-11-25).

## Next — M3: Desktop multiplataforma e release

Resultado: mesmo nível do M1 em macOS e Windows, com release público assinado.

Estado em 2026-11-05: a `v3.0.0` saiu com os primeiros instaladores Windows (`.exe` e `.msi`, sem
assinatura) ao lado dos pacotes Linux. Os jobs Windows do CI são required checks no `main`. O
Windows está em beta: tudo passa no CI e no smoke dos instaladores, mas ninguém testou à mão numa
máquina real. O macOS ainda só compila a partir do código.

Done no M3: `TAURI-016` (base do Windows, CI e instaladores) e `FILE-007` (preflight de copiar,
mover e apagar).

Estado em 2026-11-10: a `v3.1.0` leva o undo da FILE-008 para cópia, sobrescrita, movimentação e
lixeira no core, no CLI e na web UI, e cópias que não perdem dados quando canceladas ou com falha.

| Ordem | Task | O que falta | Sprint |
|---|---|---|---|
| 1 | TAURI-012 | Matriz de regressão à mão: rodada no Windows com a `v3.0.0` e as checagens que sobraram da TAURI-016; macOS depois | S05 |
| 2 | CORE-014 | "Abrir com" e abrir arquivos no Windows e no macOS, conferidos na rodada da TAURI-012 | S05 |
| 3 | REL-001 | Assinatura: updater do Tauri, Authenticode, Apple, Google Play; AUR só em tags estáveis | S05 |
| 4 | TAURI-017 | Repetir como administrador no Windows via UAC; espera o certificado da REL-001 | — |
| 5 | CLI-002 | `wing update` não acha binários nos releases do WingDrive (Done em 2026-11-05) | — |
| 6 | FILE-008 | Done em 2026-11-12: todo undo do Explorer passa pelo journal do core (#378) | — |
| 7 | TAURI-000 | Fecha quando TAURI-012 e REL-001 fecharem | S05 |

O M3 fecha com a TAURI-012 Done em Windows e macOS, e um release assinado em todas as plataformas
que têm credencial.

### Cobertura de testes no M2 e M3

Por solicitação do usuário em 2026-11-03, o épico `COV-00` amplia a cobertura do core,
desktop e CLI usando a infraestrutura da DEV-006. [Plano e metas](testing/README.md).
As tasks `COV-01` a `COV-12` têm dependências e critérios verificáveis. Em 2026-11-10,
COV-01 a COV-04, COV-06 e COV-07 estão In Progress, com suítes no `main` (#343, #344, #358, #359)
e sem a medição LCOV que fecha cada meta; as demais ficam no backlog, sem alterar o compromisso
do S04/S05 até haver capacidade no planning.

No M2 entram os contratos e regressões críticas; o M3 fecha integração e gates com metas de
50% no core, 75% no desktop, 60% no CLI e 60% na interface React, além de 80% nas linhas de
produção alteradas. São metas futuras, verificadas pelo relatório completo, sem excluir
código para elevar a média. O épico fecha com todas as filhas Done e evidência no main.

## Later — M4: Integração entre dispositivos

Pairing, Spacedrop, library sync e file sync confiáveis entre desktops.

- `FSYNC-000` File Sync
- `SHARE-001` arquitetura de compartilhamento
- Revalidar `NET-000` e `LSYNC-000` (marcados Done no upstream) em uso real

## Later — M5: Opcionais

Cloud (`CLOUD-000`, `CLOUD-003`), extensões (`PLUG-000`), Spacebot (`TAURI-004`), IA (`AI-000`),
mobile (`RES-000`, `SEC-009`). Só entram depois de M4, salvo decisão explícita.

Exceção decidida em 2026-11-12: integrações de nuvem de terceiros (épico `INTG-000`). A integração
Google (`CLOUD-004`: Drive como volume, Google Fotos, sincronização de pastas com a nuvem) entrou no
S05 a pedido do autor e fecha a parte de cópia da `FILE-003`. Para qualquer usuário conectar só com
login faltam INTG-001 a INTG-005 (cliente OAuth embutido, política de privacidade, verificação do
Google, validação real e reconexão), planejadas no [S06](sprints/S06.md). Depois vêm o framework de
provedores (INTG-020) e Dropbox, OneDrive, Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA,
Proton Drive, armazenamento de objetos, SFTP e SMB (INTG-021 a INTG-031).

Em 2026-11-13 o login só com o botão funcionou com o Google real: cada provedor tem dois modos,
"Conta" (cliente do WingDrive embutido no build, como o GNOME Online Accounts faz) e cliente próprio
(Google Cloud ou Microsoft Entra), escolhidos num catálogo de cards em Configurações > Integrações
(INTG-010). OneDrive (INTG-022) já entra por login no navegador; falta registrar o app do WingDrive
na Microsoft. INTG-001 a INTG-004 andaram no S05.

Exceção decidida em 2026-10-17: o épico mobile `MOB-000` (Android funcional, loop de dev rápido,
paridade com o desktop) foi puxado para agora. MOB-001 a MOB-005 entram no S03.

## Backlog herdado do upstream

Tasks criadas no upstream que ainda estavam abertas; desde 2026-11-13 todas têm `bernardopg` como responsável. Na auditoria de
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
| 2026-11-13 | Release `v3.3.0` (#393): login no Google com um botão pelo cliente OAuth do WingDrive embutido no build (secrets do repositório), modo com cliente próprio, OneDrive por login no navegador, catálogo de integrações, `/privacy` e `/service-terms` |
| 2026-11-13 | Integrações: modos "Conta" e cliente próprio com catálogo de cards (INTG-010), OneDrive por login no navegador (INTG-022), projeto e cliente OAuth do WingDrive no Google com login real funcionando (INTG-001, INTG-004), `/privacy` e `/service-terms` no site (INTG-002) |
| 2026-11-13 | Release `v3.2.0` (#383 a #389): integração Google com Drive e Fotos em várias contas (CLOUD-004), operações e edição em volumes de nuvem, cloud sync, COV-07 Done com 91,6% de cobertura do shell desktop e os bugs achados corrigidos, `sdPath` para `wdPath` (BRAND-003) |
| 2026-11-13 | Todas as tasks com responsável `bernardopg`, milestone (M0 para as concluídas no upstream) e sprint; sprints S07 a S14 planejados |
| 2026-11-12 | Release `v3.1.1` (#368 a #381): portal `FileChooser` (DESK-051), undo de renomear e novos itens no Tauri (FILE-008), correções DESK-090 a DESK-096, CORE-020, CLI-010, cerca de 175 testes e atualizações do Dependabot |
| 2026-11-12 | PRs #373 a #379: site com deploy automático e Discord em inglês; FILE-008 Done; DESK-051 com o backend do portal `FileChooser` (falta conferir com navegadores e no pacote); revalidação do S03/S04 achou DESK-090 a DESK-096; coverage com CORE-020, CLI-010 e DESK-060 |
| 2026-11-10 | Release `v3.1.0` em preparação (#347 a #360): FILE-008 com undo no core, CLI e web UI (falta o lado Tauri); COV-03 sem perda de dados em cópia cancelada; MOB-013 e MOB-014 Done; SQLite embutido atualizado (RUSTSEC-2026-0459); COV-04, COV-06 e COV-07 In Progress com 79 testes e sete bugs corrigidos |
| 2026-11-05 | M2: WATCH-006, DESK-016 e DESK-013 Done; revisão das 2160 frases pt-BR (46 correções); dez rodadas da DESK-043 com a `v3.0.0` (estáveis, 4 de 10 abaixo de 550 MiB). Falta a DESK-043 para fechar o DESK-000 |
| 2026-11-05 | Release `v3.0.0` com os primeiros instaladores Windows (#331 a #334). TAURI-016 Done, jobs Windows viram required checks (#336) e o README marca o Windows como beta (#335). Sobras da TAURI-016 distribuídas: checagens à mão na TAURI-012, assinatura na REL-001, UAC na nova TAURI-017. Nova CLI-002 (`wing update` sem binários no release). Tabela do M3 com ordem |
| 2026-11-03 | COV-00 e COV-01 a COV-12: metas progressivas de cobertura, prioridade em core/desktop/CLI e critérios de evidência, sem ampliar sprints já comprometidos |
| 2026-11-02 | WingDrive independente (#310): tasks FORK-* removidas do tracker (histórico no git); NET-007 herda o critério aberto da FORK-007, REL-001 (S05) a assinatura da FORK-002, FORK-009 descartada; levantamentos do upstream em `docs/design/history/`. S01 fechado com review e retro. Checagem no pacote fecha 16 tasks do S04 e acha DESK-039 a DESK-042; release 2.4.0 |
| 2026-10-28 | #296 mergeado; S02 e S03 fechados com review e retro; sobras do S03 (DESK-021, DESK-023, DESK-024, WATCH-006, MOB-005, MOB-009) no S04; DESK-034 e as duas falhas da DESK-029 (emblema Git nos filhos e contagem da barra de status) entram no S04; DESK-016 ganha notas de desenho |
| 2026-10-28 | S04: DESK-029 a DESK-033 e DESK-035 com código pronto (PRs #296 e #298), In Progress até a checagem no build Linux; visualização rápida de PDF, texto e código (#295); pdfjs-dist 6.4 fecha o alerta Dependabot 47 (#297); DESK-034 adiada |
| 2026-10-17 | Revisão do app mobile no Moto G7 e no emulador: épico MOB-000 com 11 tasks em `.tasks/mobile/`; MOB-001 a MOB-005 puxadas para o S03 |
| 2026-10-16 | Auditoria do `.tasks/`: quadro do S03 atualizado, S04 e S05 planejados; 65 tasks herdadas do upstream estacionadas em M4/M5; EXPL-000 e SEC-008 Done; CORE-012 e CORE-013 obsoletas |
| 2026-10-15 | M2 quase fechado; candidatas DESK-015, DESK-016 e DESK-017 abertas; app mobile traduzido e com typecheck no CI |
| 2026-10-07 | Novo M2 "Gerenciador de arquivos padrão no Linux" (DESK-000) como prioridade máxima; antigos M2–M4 viram M3–M5 |
| 2026-10-06 | M1 completo: TAURI-006 e BRAND-002 Done; INDEX-010 integrada; sync multi-device corrigido |
| 2026-10-06 | FORK-004, TAURI-014 e FORK-005 Done; licença Apache-2.0; release `v2.0.0-alpha.7` |
| 2026-10-05 | Now atualizado: alpha.6 publicada no AUR; TAURI-013 dividida (TAURI-014); BRAND-002 e TAURI-006 no S02 |
| 2026-10-01 | Roadmap criado; M1 definido a partir da revisão `docs/design/history/TAURI_FILEMANAGER_REVIEW_2026-09-03.md` |
