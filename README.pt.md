<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="120" height="120" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>Um gerenciador de arquivos para todos os seus dispositivos e nuvens.</strong><br/>
  <span>Local-first, pronto para IA, sincronizado peer-to-peer para macOS, Windows, Linux, iOS e Android</span>
</p>

<p align="center">
  <a href="./LICENSE">
    <img src="https://img.shields.io/static/v1?label=License&message=Apache-2.0&color=000" />
  </a>
  <img src="https://img.shields.io/static/v1?label=Core&message=Rust&color=DEA584" />
  <img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?include_prereleases&label=Versão&color=7c3aed" />
  <img src="https://img.shields.io/static/v1?label=Plataformas&message=macOS%20%7C%20Windows%20%7C%20Linux&color=36A3FF" />
</p>

<p align="center">
  <a href="#primeiros-passos">Primeiros Passos</a> &bull;
  <a href="#novidades-alpha16">Novidades no alpha16</a> &bull;
  <a href="#plataformas">Plataformas</a> &bull;
  <a href="#roadmap">Roadmap</a> &bull;
  <a href="https://github.com/bernardopg/wingdrive/releases">Baixar</a> &bull;
  <a href="./NOTICE.md">Aviso &amp; Licença</a> &bull;
  <a href="./CONTRIBUTING.md">Contribuir</a>
</p>

---

> **WingDrive é uma continuação comunitária fork do [Spacedrive](https://github.com/spacedriveapp/spacedrive).**
>
> O desenvolvimento upstream parou, então este fork carrega o trabalho adiante de forma aberta. Preserva todo o histórico upstream e mantém o copyright original.
>
> WingDrive **não é afiliado, endossado ou suportado pela Spacedrive Technology Inc.** Não reporte problemas do WingDrive ao upstream.
>
> O código é licenciado sob **Apache-2.0**. Veja [NOTICE.md](./NOTICE.md) para detalhes completos e atribuição do fork.

---

<div align="center">

# Seus arquivos, em todos os lugares. Seus dados, seus.

WingDrive é um gerenciador de arquivos local-first que unifica seus dados em todos os dispositivos e serviços de nuvem sem lock-in de nuvem centralizada. Os arquivos ficam onde estão. O WingDrive os indexa, torna pesquisável e sincroniza peer-to-peer entre suas próprias máquinas.

</div>

---

## Novidades no 2.0.0-alpha16

Lançado em 15 de outubro de 2026. Detalhes completos no [CHANGELOG](./CHANGELOG.md).

<details>
<summary><strong>Explorer</strong></summary>

- Pastas mostram o tamanho real do conteúdo; configuração opcional "Calcular Tamanho da Pasta" para pastas não indexadas
- Arquivos com hard-link contam uma vez no tamanho da pasta e contagem de arquivos
- Abra uma pasta em uma nova janela pelo menu de contexto ou com Ctrl+N (Cmd+N no macOS)
- Renomear ou mover uma pasta em uma pasta não indexada agora atualiza a view e reindexa sem erros

</details>

<details>
<summary><strong>Desktop</strong></summary>

- Menu da bandeja segue o idioma da interface, incluindo lançamentos ocultos de autostart
- Página About e links Wingbot abrem no navegador do sistema
- Strings de interface remanescentes hard-coded estão traduzidas

</details>

<details>
<summary><strong>Dispositivos e armazenamento</strong></summary>

- Um drive desconectado é marcado como offline em vez de permanecer online

</details>

<details>
<summary><strong>Confiabilidade</strong></summary>

- Desligamento dentro do debounce do watcher da biblioteca não reabre mais a biblioteca nem deixa arquivos de lock para trás
- Migração repara pastas e arquivos ausentes em queries de descendentes

</details>

<details>
<summary><strong>Mobile</strong></summary>

- Disponível em inglês e português do Brasil
- Excluir uma biblioteca, selecionar uma nova biblioteca após criação, locais fixos aparecem no Browse, conexões Tailscale são rotuladas

</details>

---

## O que é o WingDrive?

WingDrive é uma plataforma de dados cross-device. Indexe arquivos, e-mails, notas e fontes externas. Pesquise tudo. Sincronize via P2P. Mantenha agentes de IA seguros com triagem integrada.

- **Identidade de conteúdo** — cada arquivo recebe um hash BLAKE3. O mesmo arquivo em dois dispositivos produz o mesmo hash. O WingDrive rastreia redundância e deduplicação em todas as suas máquinas.
- **Cross-device** — veja todos os seus arquivos em todos os seus dispositivos em um só lugar. Arquivos em dispositivos desconectados permanecem no índice e aparecem como offline.
- **Sincronização P2P** — dispositivos se conectam diretamente via Iroh/QUIC. Sem servidores, sem nuvem, sem ponto único de falha. Metadados sincronizam entre dispositivos. Arquivos permanecem onde estão.
- **Volumes de nuvem** — indexe S3, Google Drive, Dropbox, OneDrive, Azure e GCS como volumes de primeira classe junto ao armazenamento local.
- **Nove views** — grid, lista, colunas, mídia, tamanho, recentes, pesquisa, conhecimento e splat. QuickPreview para vídeo, áudio, código, documentos, 3D e imagens.
- **Local-first** — tudo roda na sua máquina. Nenhum dado deixa seu dispositivo a menos que você escolha sincronizar entre seus próprios dispositivos.

### Isso é um substituto para o Finder ou Explorer?

Não. O WingDrive fica acima do gerenciador de arquivos do seu SO e adiciona capacidades que Finder/Explorer não têm:

- **Portal para tudo** — pesquise e navegue por arquivos em discos locais, drives externos, NAS, armazenamento em nuvem e fontes de dados arquivadas de uma única interface.
- **Superfície operacional para arquivos** — identidade de conteúdo, sidecars, artefatos derivados, metadados ricos, sincronização e consciência cross-device integrados ao modelo central.
- **Embutível e compartilhável** — execute como app desktop, servidor headless, serviço de arquivo hospedado, ou embarque a interface e APIs em outros produtos.
- **Pronto para IA por design** — pipelines de indexação e análise preparam os dados com antecedência em vez de dar acesso de shell bruto aos agentes.
- **Modelo de acesso mais seguro** — roteie IA e automação através de APIs estruturadas, permissões e camadas de processamento em vez de operações diretas de arquivo.

Você ainda usa seu SO para interações de baixo nível com arquivos. O WingDrive adiciona a camada cross-platform, cross-device, cloud-aware e amigável à automação por cima.

---

## Veja em Ação

<div align="center">
  <img src="docs/public/WingColumnView.webp" alt="Column view file explorer" width="600"/>
  <p><em>Column view — navegue pela sua biblioteca entre dispositivos</em></p>
</div>

<div align="center" style="margin-top: 2rem;">
  <img src="docs/public/WingGridView.webp" alt="Grid view" width="300"/>
  <img src="docs/public/WingListView.webp" alt="List view" width="300"/>
  <img src="docs/public/WingMediaView.webp" alt="Media view" width="300"/>
  <img src="docs/public/WingSizeView.webp" alt="Size view" width="300"/>
  <p><em>Grid, lista, mídia e views de tamanho</em></p>
</div>

---

## Por que WingDrive?

A computação foi projetada para dispositivos únicos. Gerenciadores de arquivos como Finder e Explorer assumem que seus dados vivem no computador na sua frente. A mudança para multi-device nos forçou a ecossistemas de nuvem onde conveniência exige abrir mão da propriedade dos dados.

A IA está acelerando esta tendência. Serviços de nuvem oferecem busca semântica e análise inteligente, mas apenas se você enviar seus arquivos para a infraestrutura deles. O WingDrive constrói a arquitetura para IA local no hardware que você já possui, em dados que nunca saem do seu controle.

| Em vez de | O WingDrive oferece |
|-----------|---------------------|
| Arquivos espalhados entre dispositivos e nuvens | Uma visão unificada de tudo |
| Enviar para a nuvem para pesquisar | Indexação local que permanece na sua máquina |
| Lock-in de fornecedor | Open source, Apache-2.0, sem lock-in |
| Processamento de IA em caixa-preta | APIs estruturadas com permissões e triagem |
| Servidores de sincronização centralizados | Sincronização peer-to-peer entre seus próprios dispositivos |

---

## Plataformas

| Plataforma | Status | Notas |
|------------|--------|-------|
| macOS | Suportado | App desktop nativo |
| Linux | Suportado | App desktop nativo, AppImage e DEB |
| Windows | Em breve no alpha17 | App desktop em progresso |
| iOS | Em breve | Protótipo React Native |
| Android | Em breve | Protótipo React Native |

---

## Guia de Instalação

O WingDrive requer [Rust](https://rustup.rs/) 1.89+, [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just), e Python 3.9+ (para adapters).

### Opção 1: Baixar release pré-compilada (recomendado)

Obtenha a última build em [GitHub Releases](https://github.com/bernardopg/wingdrive/releases):

- **Linux:** AppImage ou pacote Debian
- **macOS:** bundle nativo em breve
- **Docker:** `ghcr.io/bernardopg/wingdrive/server` para implantações headless

### Opção 2: Sistemas baseados em Arch (AUR)

O WingDrive está disponível no AUR para sistemas baseados em Arch (Arch Linux, Manjaro, EndeavourOS, etc.).

<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/badge/AUR-wingdrive--bin-0090FF?logo=arch-linux&logoColor=white" alt="AUR wingdrive-bin"/>
  </a>
</p>

Instale com um helper do AUR (ex: `yay` ou `paru`):

```bash
yay -S wingdrive-bin
```

Ou instale manualmente:

```bash
git clone https://aur.archlinux.org/wingdrive-bin.git
cd wingdrive-bin
makepkg -si
```

### Opção 3: Build a partir do código-fonte

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # bun install + native deps + cargo config
just dev-desktop  # lança o app desktop (inicia o daemon automaticamente)
just test         # executa todos os testes do workspace
```

Então use o CLI:

```bash
cargo run --bin wing -- <command>
```

### Primeira execução

1. Inicie o app desktop.
2. Adicione sua primeira localização de biblioteca (uma pasta, drive externo ou volume de nuvem).
3. O WingDrive indexa o conteúdo e o torna pesquisável em todos os seus dispositivos.

---

## Privacidade e Segurança

O WingDrive é local-first. Seus dados permanecem nos seus dispositivos.

- **Criptografia Ponta-a-Ponta** — todo tráfego P2P criptografado via QUIC/TLS
- **Criptografia em Repouso** — bibliotecas podem ser criptografadas em disco (SQLCipher)
- **Sem Telemetria** — zero rastreamento ou analytics
- **Auto-hospedável** — execute seus próprios servidores de retransmissão
- **Soberania dos Dados** — você controla onde seus dados vivem

Integração opcional de nuvem está disponível para backup e acesso remoto, mas nunca é obrigatória. O serviço de nuvem executa o core não modificado como um dispositivo P2P padrão — sem privilégios especiais.

### Triagem de Segurança

Quando habilitada, cada registro passa por um pipeline de segurança antes de se tornar pesquisável:

- **Prompt Guard 2** — classificador local detecta injeção de prompt em e-mails, mensagens e documentos antes de entrarem no índice.
- **Níveis de Confiança** — conteúdo autoral (suas notas) recebe triagem balanceada; conteúdo externo (caixa de entrada de e-mail) recebe triagem rígida.
- **Sistema de Quarentena** — registros sinalizados são excluídos de queries de agentes de IA e são revisáveis no app desktop.
- **Cerca de Conteúdo** — resultados de busca incluem metadados de confiança para que agentes saibam o que é seguro versus não confiável.

Nenhuma outra ferramenta de dados local triagem conteúdo indexado antes de expô-lo à IA.

---

## Arquitetura

O núcleo é construído sobre quatro princípios:

1. **Virtual Distributed Filesystem (VDFS)** — arquivos e pastas se tornam objetos de primeira classe com metadados ricos, independentes da localização física. Cada arquivo recebe um endereço universal (`WingPath`) que funciona entre dispositivos.
2. **Content Identity System** — hashing adaptativo (BLAKE3 com amostragem estratégica para arquivos grandes) cria uma impressão digital única para cada conteúdo. Permite deduplicação, rastreamento de redundância e operações baseadas em conteúdo.
3. **Ações Transacionais** — cada operação de arquivo pode ser pré-visualizada antes da execução. Veja economia de espaço, conflitos e tempo estimado, então aprove ou cancele. Operações tornam-se jobs duráveis que sobrevivem a interrupções de rede e reinicializações de dispositivo.
4. **Sincronização Leaderless** — sincronização peer-to-peer sem coordenadores centrais. Dados específicos de dispositivo usam replicação de estado. Metadados compartilhados usam um log ordenado por HLC com resolução determinística de conflitos.

A implementação é um único crate Rust com arquitetura CQRS/DDD. Cada operação (cópia de arquivo, criação de tag, consulta de busca) é uma ação ou consulta registrada com entrada/saída tipada que gera automaticamente tipos TypeScript para o frontend.

| Componente | Tecnologia |
|------------|------------|
| Linguagem | Rust |
| Runtime assíncrono | Tokio |
| Banco de dados | SQLite (SeaORM + sqlx) |
| P2P | Iroh (QUIC, hole-punching, descoberta local) |
| Hash de conteúdo | BLAKE3 |
| Busca vetorial | LanceDB + FastEmbed |
| Armazenamento em nuvem | OpenDAL |
| Criptografia | Ed25519, X25519, ChaCha20-Poly1305, AES-GCM |
| Mídia | FFmpeg, libheif, Pdfium, Whisper |
| Desktop | Tauri 2 |
| Mobile | React Native + Expo |
| Frontend | React 19, Vite, TanStack Query, Tailwind CSS v4 |
| Sistema de design | Workspaces vendados `@wingdrive/primitives`, `@wingdrive/tokens`, e `@wingdrive/ai` |
| Geração de tipos | Specta |

```
wingdrive/
├── core/                  # Motor Rust (CQRS/DDD)
├── apps/
│   ├── tauri/             # App desktop (macOS, Windows, Linux)
│   ├── mobile/            # React Native (iOS, Android)
│   ├── cli/               # CLI e daemon
│   ├── server/            # Servidor headless
│   └── web/               # Cliente de navegador
├── packages/
│   ├── interface/         # UI React compartilhada
│   ├── ts-client/         # Cliente TypeScript gerado automaticamente
│   ├── wingdrive-primitives/ # Biblioteca de componentes
│   ├── wingdrive-tokens/  # Tokens de design
│   ├── wingdrive-ai/      # Componentes de interface para agentes
│   └── assets/            # Ícones, imagens, SVGs
├── crates/                # Crates Rust autônomos (ffmpeg, crypto, etc.)
├── adapters/              # Adapters baseados em script para fontes de dados
└── schemas/               # Esquemas de dados TOML
```

---

## Roadmap

- **alpha17** — suporte para desktop Windows
- **Releases futuras** — app iOS, app Android, edição servidor para implantações headless, Extension SDK (API estável), integração de IA com modelos locais

Veja a [página de releases](https://github.com/bernardopg/wingdrive/releases) para o roadmap público e o [board de tarefas](/.tasks/) para rastreamento interno.

---

## Comunidade

- **Discord** — [discord.gg/gTaF2Z44f5](https://discord.gg/gTaF2Z44f5)
- **GitHub Issues** — [reporte bugs ou solicite recursos](https://github.com/bernardopg/wingdrive/issues/new/choose)
- **Whitepaper** — [docs/overview/whitepaper.mdx](./docs/overview/whitepaper.mdx) (em andamento)
- **Docs de arquitetura** — comece com [core/architecture](./docs/core/architecture.mdx)

---

## Contribuir

Contribuições são bem-vindas de qualquer pessoa. Abra uma issue ou um pull request.

- **[Guia de Contribuição](CONTRIBUTING.md)**
- **[Guia de Adapters](docs/ADAPTERS.md)** — escreva um adapter de fonte de dados
- Os workspaces de sistema de design vivem neste repositório sob `packages/wingdrive-*`.

Ao abrir um pull request você concorda que sua contribuição é licenciada sob Apache-2.0, os mesmos termos do restante do projeto.

---

## Licença

**Apache-2.0** — veja [LICENSE](./LICENSE). Copyright 2026 Spacedrive Technology Inc. e contribuidores do WingDrive. Ícones de tipo de arquivo em `packages/assets/icons` permanecem sob GPL-3.0-only; veja [NOTICE.md](./NOTICE.md).

Converte para Apache-2.0 em 2028 sob a Grant of Future License.

Detalhes completos e atribuição do fork: [NOTICE.md](./NOTICE.md).