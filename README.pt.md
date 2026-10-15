<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="100" height="100" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>Um gerenciador de arquivos para todos os seus dispositivos e nuvens.</strong><br/>
  <span>Local-first · pronto para IA · peer-to-peer · macOS · Windows · Linux · iOS · Android</span>
</p>

<p align="center">
  <a href="https://github.com/bernardopg/wingdrive/releases">
    <img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?include_prereleases&label=Versão&color=7c3aed" alt="Versão"/>
  </a>
  <img src="https://img.shields.io/static/v1?label=License&message=Apache-2.0&color=000" alt="Licença"/>
  <img src="https://img.shields.io/static/v1?label=Core&message=Rust&color=DEA584" alt="Core"/>
  <img src="https://img.shields.io/static/v1?label=Plataformas&message=macOS%20%7C%20Windows%20%7C%20Linux&color=36A3FF" alt="Plataformas"/>
</p>

---

<div align="center">

# Seus arquivos, em todos os lugares. Seus dados, seus.

O WingDrive unifica seus dados em todos os dispositivos e nuvens — sem lock-in centralizado. Os arquivos ficam onde estão. O WingDrive os indexa, torna pesquisável e sincroniza peer-to-peer entre suas próprias máquinas.

</div>

---

## ✨ O que é o WingDrive?

Uma plataforma de dados cross-device. Indexe arquivos, e-mails, notas e fontes externas. Pesquise tudo. Sincronize via P2P.

- 🔑 **Identidade de conteúdo** — hash BLAKE3 em cada arquivo. Mesmo arquivo em dois dispositivos = mesmo hash. Deduplicação em todas as suas máquinas.
- 🖥️ **Cross-device** — veja todos os seus arquivos em todos os dispositivos em um só lugar. Dispositivos offline permanecem no índice.
- 🔄 **Sincronização P2P** — direto entre dispositivos via Iroh/QUIC. Sem servidores, sem nuvem, sem ponto único de falha.
- ☁️ **Volumes de nuvem** — S3, Google Drive, Dropbox, OneDrive, Azure, GCS como volumes de primeira classe.
- 📱 **Nove views** — grid, lista, colunas, mídia, tamanho, recentes, pesquisa, conhecimento, splat. QuickPreview para vídeo, áudio, código, documentos, 3D, imagens.
- 🏠 **Local-first** — tudo roda na sua máquina. Nenhum dado deixa seu dispositivo a menos que você escolha sincronizar.

---

## 🚀 Novidades no 2.0.0-alpha16

Lançado em 15 de outubro de 2026.

<details>
<summary><strong>📁 Explorer</strong></summary>
Pastas mostram tamanho real do conteúdo · hard-links contam uma vez · abrir pasta em nova janela (Ctrl+N / Cmd+N) · correção de renomear/mover em pastas não indexadas
</details>

<details>
<summary><strong>🖥️ Desktop</strong></summary>
Menu da bandeja segue o idioma da interface · links About/Wingbot abrem no navegador do sistema · strings restantes traduzidas
</details>

<details>
<summary><strong>💾 Dispositivos & Armazenamento</strong></summary>
Drives desconectados marcados como offline em vez de permanecerem online
</details>

<details>
<summary><strong>🛡️ Confiabilidade</strong></summary>
Correção de shutdown no debounce do watcher da biblioteca · migração repara arquivos ausentes em queries de descendentes
</details>

<details>
<summary><strong>📱 Mobile</strong></summary>
Inglês e português do Brasil · excluir biblioteca · locais fixos no Browse · rótulos Tailscale
</details>

> [Changelog completo →](./docs/readme/changelog-alpha16.md)

---

## 🖼️ Veja em Ação

<div align="center">
  <img src="docs/public/WingColumnView.webp" alt="Column view" width="300"/>
  <img src="docs/public/WingGridView.webp" alt="Grid view" width="300"/>
  <img src="docs/public/WingListView.webp" alt="List view" width="300"/>
  <img src="docs/public/WingMediaView.webp" alt="Media view" width="300"/>
  <img src="docs/public/WingSizeView.webp" alt="Size view" width="300"/>
</div>

---

## 🎯 Por que WingDrive?

| Em vez de | O WingDrive oferece |
|-----------|---------------------|
| Arquivos espalhados entre dispositivos e nuvens | Uma visão unificada de tudo |
| Enviar para a nuvem para pesquisar | Indexação local na sua máquina |
| Lock-in de fornecedor | Open source, Apache-2.0, sem lock-in |
| Processamento de IA em caixa-preta | APIs estruturadas com permissões e triagem |
| Servidores de sincronização centralizados | Sincronização peer-to-peer entre seus próprios dispositivos |

---

## 📦 Plataformas

| Plataforma | Status |
|------------|--------|
| macOS | ✅ Suportado |
| Linux | ✅ Suportado (AppImage, DEB) |
| Windows | 🔜 Em breve no alpha17 |
| iOS | 🔜 Em breve |
| Android | 🔜 Em breve |

---

## 🛠️ Instalação

> Detalhes completos → [Guia de Instalação](./docs/readme/installation.md)

**Release pré-compilada (recomendado):**
[GitHub Releases](https://github.com/bernardopg/wingdrive/releases) — AppImage, Debian, Docker.

**Sistemas baseados em Arch (AUR):**
<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/badge/AUR-wingdrive--bin-0090FF?logo=arch-linux&logoColor=white" alt="AUR wingdrive-bin"/>
  </a>
</p>

```bash
yay -S wingdrive-bin
```

**Build a partir do código-fonte:**
```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive
just setup
just dev-desktop
```

---

## 📚 Arquitetura

> Referência completa → [Arquitetura](./docs/readme/architecture.md)

Baseada em quatro princípios: **VDFS**, **Identidade de Conteúdo**, **Ações Transacionais**, **Sincronização Leaderless**.  
Stack: Rust · Tokio · SQLite · Iroh/QUIC · BLAKE3 · LanceDB · OpenDAL · Tauri 2 · React 19.

---

## 🔒 Privacidade e Segurança

- **Criptografia Ponta-a-Ponta** — tráfego P2P via QUIC/TLS
- **Criptografia em Repouso** — bibliotecas criptografadas em disco (SQLCipher)
- **Sem Telemetria** — zero rastreamento ou analytics
- **Soberania dos Dados** — você controla onde seus dados vivem

Triagem de segurança: Prompt Guard 2, níveis de confiança, quarentena, cerca de conteúdo.

---

## 🗺️ Roadmap

- **alpha17** — suporte para desktop Windows
- **Futuro** — app iOS, app Android, edição servidor, Extension SDK (API estável), modelos de IA locais

[Releases →](https://github.com/bernardopg/wingdrive/releases)

---

## 🤝 Comunidade

- **Discord** — [discord.gg/gTaF2Z44f5](https://discord.gg/gTaF2Z44f5)
- **Issues** — [reporte bugs / solicite recursos](https://github.com/bernardopg/wingdrive/issues/new/choose)
- **Whitepaper** — [docs/overview/whitepaper.mdx](./docs/overview/whitepaper.mdx)

---

## 📄 Licença

**Apache-2.0** — veja [LICENSE](./LICENSE). Ícones de tipo de arquivo em `packages/assets/icons` permanecem sob GPL-3.0-only; veja [NOTICE.md](./NOTICE.md).
