<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="100" height="100" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>Um gerenciador de arquivos local-first que indexa todos os seus discos.</strong><br/>
  <span>Core em Rust · Desktop Tauri · CLI · Servidor web · Sync peer-to-peer</span>
</p>

<p align="center">
  <a href="https://github.com/bernardopg/wingdrive/releases">
    <img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?include_prereleases&label=Versão&color=7c3aed" alt="Versão"/>
  </a>
  <a href="https://github.com/bernardopg/wingdrive/actions/workflows/ci.yml">
    <img src="https://github.com/bernardopg/wingdrive/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"/>
  </a>
  <img src="https://img.shields.io/static/v1?label=Licença&message=Apache-2.0&color=000" alt="Licença"/>
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/aur/version/wingdrive-bin?label=AUR&color=0090FF" alt="AUR"/>
  </a>
</p>

<p align="center"><a href="./README.md">English</a> · Português</p>

---

O WingDrive é um gerenciador de arquivos desktop com um daemon que indexa suas pastas, discos
externos e volumes de nuvem numa biblioteca só. Os arquivos ficam onde estão. O índice torna tudo
pesquisável, dá a cada arquivo um hash de conteúdo e deixa seus próprios dispositivos
sincronizarem a biblioteca peer-to-peer, sem servidor central.

> **Status: alpha.** Hoje a plataforma suportada é o Linux, e a meta atual é o WingDrive substituir
> Dolphin, Nautilus ou Yazi como gerenciador de arquivos padrão no Linux. macOS e Windows compilam a
> partir do código-fonte; releases prontas e assinadas para eles estão planejadas.

## Recursos

**Gerenciador de arquivos**

- Visões em grade, lista, colunas, mídia e tamanho, com QuickPreview para imagens, vídeo, áudio,
  código e documentos.
- Copiar, mover, renomear, duplicar, apagar, e Copiar para… / Mover para… uma pasta escolhida, com
  tratamento de conflitos e desfazer para renomear, mover e pastas novas.
- Abre pastas pelo `xdg-open` e pelo "Mostrar na pasta" de outros apps
  (`org.freedesktop.FileManager1`), fica na bandeja e inicia com a sessão.
- Abrir com, abrir terminal aqui, novo arquivo vazio, links simbólicos, permissões e dono, compactar
  e extrair, checksum SHA-256, selecionar por padrão e inverter seleção.
- Tamanho real das pastas, inclusive das que ainda não foram indexadas.

**Índice e busca**

- Cada arquivo ganha uma identidade de conteúdo BLAKE3, então o mesmo arquivo em dois dispositivos é
  reconhecido como um só.
- A busca refina por tipo, extensão, tamanho, data e arquivos ocultos, com contagens sobre o
  resultado inteiro.
- Um watcher do sistema de arquivos mantém o índice em dia; discos desconectados aparecem offline.

**Entre dispositivos**

- Pareamento e sync da biblioteca entre seus próprios dispositivos via Iroh/QUIC.
- Volumes de nuvem via OpenDAL (S3, Google Drive, Dropbox, OneDrive, Azure Blob, GCS).
- O `wing-server` serve a mesma interface no navegador para máquinas sem tela.

**Interface**

- Inglês e português do Brasil. As traduções ficam no Crowdin.
- Temas embutidos, entre eles Catppuccin, Gruvbox, Solarized, Rosé Pine, Dracula e Tokyo Night.

<div align="center">
  <img src="docs/public/WingColumnView.webp" alt="Visão em colunas" width="300"/>
  <img src="docs/public/WingGridView.webp" alt="Visão em grade" width="300"/>
  <img src="docs/public/WingListView.webp" alt="Visão em lista" width="300"/>
  <img src="docs/public/WingMediaView.webp" alt="Visão de mídia" width="300"/>
  <img src="docs/public/WingSizeView.webp" alt="Visão de tamanho" width="300"/>
</div>

## Novidades

Cada release está no [CHANGELOG.md](./CHANGELOG.md) (em inglês), e a
[página de Releases](https://github.com/bernardopg/wingdrive/releases) tem os downloads de cada tag.

## Plataformas

| Plataforma | Status |
|------------|--------|
| Linux (x86_64) | Suportado: AppImage, pacote Debian, AUR (`wingdrive-bin`) |
| Servidor sem tela | `wing-server` com a interface web, imagem Docker no GHCR |
| macOS | Compila do código-fonte; release pronta planejada (marco M3) |
| Windows | Compila do código-fonte; release pronta planejada (marco M3) |
| iOS, Android | Em desenvolvimento (`apps/mobile`), sem release ainda |

## Instalação

Guia completo: [docs/readme/installation.md](./docs/readme/installation.md).

**Arch Linux e derivados (AUR):**

```bash
yay -S wingdrive-bin
```

**Outras distribuições Linux:** baixe o AppImage ou o `.deb` em
[GitHub Releases](https://github.com/bernardopg/wingdrive/releases). Cada release traz um arquivo
`SHA256SUMS`.

**Servidor sem tela** (interface web na porta 8080, dados no volume `wingdrive-data`; veja
[Linux deployment](./docs/cli/linux-deployment.mdx)):

```bash
docker run -d -p 8080:8080 -p 7373:7373 \
  -v wingdrive-data:/data -v /caminho/dos/arquivos:/mnt/files:ro \
  ghcr.io/bernardopg/wingdrive/server
```

**Do código-fonte** (Rust do `rust-toolchain.toml`, [Bun](https://bun.sh) 1.3+,
[just](https://github.com/casey/just)):

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive
just setup         # dependências nativas e .cargo/config.toml
just dev-desktop   # app desktop; sobe o próprio daemon
just cli --help    # o CLI `wing`
```

## Arquitetura

Um processo `wing-daemon` guarda todo o estado. O app desktop, o CLI `wing`, a interface web e o
`wing-server` são clientes finos que falam JSON-RPC com ele. O app mobile embute o mesmo core como
biblioteca.

Stack: Rust, Tokio, SQLite (SeaORM), Iroh/QUIC, BLAKE3, OpenDAL, Tauri 2, React 19.
Detalhes: [docs/readme/architecture.md](./docs/readme/architecture.md) e a documentação de cada
subsistema em [`docs/core`](./docs/core).

## Privacidade e segurança

- **Local-first:** indexação, miniaturas e busca rodam na sua máquina. Sem telemetria nem analytics.
- **Transporte criptografado:** o tráfego peer-to-peer entre dispositivos pareados usa QUIC com TLS.
- **Seu armazenamento:** o WingDrive lê seus arquivos no lugar; não envia nada para onde você não
  adicionou como volume.
- **Operações destrutivas protegidas:** o apagamento seguro recusa sistemas de arquivos copy-on-write,
  cópias e movimentos recusam escrever dentro da própria origem, e o daemon não ejeta montagens do
  sistema.

Criptografia da biblioteca em repouso ainda não existe. Reporte vulnerabilidades como descrito no
[SECURITY.md](./SECURITY.md).

## Roadmap

| Marco | Objetivo |
|-------|----------|
| M1 (feito) | Gerenciador de arquivos desktop no Linux, a partir de um bundle de produção |
| M2 (agora) | Gerenciador de arquivos padrão no Linux: uso diário sem voltar a outro |
| M3 | Releases para macOS e Windows, instaladores assinados, preflight de cópia e desfazer ampliado |
| M4 | Recursos entre dispositivos: sync de arquivos, compartilhamento público, Wingdrop |

Detalhes em [.tasks/ROADMAP.md](./.tasks/ROADMAP.md).

## Contribuindo

Leia o [CONTRIBUTING.md](./CONTRIBUTING.md) e o [AGENTS.md](./AGENTS.md) primeiro. Rode
`just ci-local` antes de dar push; ele roda os mesmos gates da CI. Bugs e pedidos de recurso vão para
as [Issues](https://github.com/bernardopg/wingdrive/issues/new/choose); dúvidas, para o
[Discord](https://discord.gg/gTaF2Z44f5).

## Licença

**Apache-2.0**, veja [LICENSE](./LICENSE). Os ícones de tipo de arquivo em `packages/assets/icons`
continuam sob GPL-3.0-only; veja [NOTICE.md](./NOTICE.md).
