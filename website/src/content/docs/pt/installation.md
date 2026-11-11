---
title: Instalação
description: Instale o WingDrive no Linux, Windows ou em um servidor, ou compile-o a partir do código-fonte.
order: 2
group: Início
---

Obtenha pacotes pré-compilados na [página de download](/pt/download/) ou no [GitHub Releases](https://github.com/bernardopg/wingdrive/releases/latest). Cada versão inclui arquivos `SHA256SUMS` para verificar o download.

## Linux

Linux (x86_64) é a plataforma compatível.

### AppImage

Funciona em qualquer distribuição sem instalar nada no sistema:

```bash
chmod +x WingDrive-*-x86_64.AppImage
./WingDrive-*-x86_64.AppImage
```

### Debian, Ubuntu e derivados

```bash
sudo apt install ./WingDrive-*-amd64.deb
```

### Arch Linux e derivados (AUR)

Dois pacotes instalam o mesmo AppImage da versão lançada e entram em conflito entre si:

- `wingdrive-bin` acompanha apenas versões estáveis.
- `wingdrive-nightly` acompanha todas as versões, incluindo alpha, beta e rc. Uma versão estável também o atualiza, então ele nunca fica atrás de `wingdrive-bin`.

```bash
yay -S wingdrive-bin        # ou: yay -S wingdrive-nightly
```

Sem um helper para AUR:

```bash
git clone https://aur.archlinux.org/wingdrive-bin.git
cd wingdrive-bin
makepkg -si
```

## Windows 10 e 11 (beta)

Baixe o instalador NSIS (`.exe`) ou o `.msi` para x64. Os instaladores ainda não são assinados, então o SmartScreen exibe um aviso na primeira execução: selecione **More info** e depois **Run anyway**. Confira o arquivo com `SHA256SUMS-windows`:

```powershell
Get-FileHash .\WingDrive-*-x64-setup.exe -Algorithm SHA256
```

## macOS

Versões pré-compiladas e assinadas estão planejadas para o marco M3. Até lá, compile a partir do código-fonte (abaixo).

## Servidor sem interface gráfica

Execute o daemon com sua interface web em um NAS, servidor doméstico ou VPS:

```bash
docker run -d --name wingdrive \
  -p 8080:8080 -p 7373:7373 \
  -v wingdrive-data:/data \
  -e WING_AUTH=admin:change-me \
  ghcr.io/bernardopg/wingdrive/server:latest
```

Consulte [Auto-hospedagem](/pt/docs/self-hosting/) para configurações com Compose, systemd e proxy reverso.

## Compile a partir do código-fonte

Você precisa da toolchain Rust fixada em `rust-toolchain.toml` (instalada pelo [rustup](https://rustup.rs/)), [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just) e Python 3.9+ (para adaptadores).

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # dependências nativas + configuração do cargo
just dev-desktop  # aplicativo desktop; inicia o daemon para você
```

Outras receitas úteis:

```bash
just dev-daemon   # somente o daemon, com suporte a ffmpeg e HEIF
just dev-server   # servidor sem interface gráfica + interface web
just cli status   # executa a CLI wing a partir do código-fonte
just test         # executa os testes do workspace
```

## Verifique um download

```bash
sha256sum -c SHA256SUMS --ignore-missing
```
