---
title: Instalação
description: Instale o WingDrive no Linux, Windows, Android ou em um servidor, ou compile-o a partir do código-fonte.
order: 2
group: Início
---

Obtenha pacotes pré-compilados na [página de download](/pt/download/) ou no [GitHub Releases](https://github.com/bernardopg/wingdrive/releases/latest). Cada versão inclui um único arquivo `SHA256SUMS` para verificar qualquer download.

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

### Flatpak

A versão também traz um pacote Flatpak no runtime GNOME 51:

```bash
flatpak install --user WingDrive-*-x86_64.flatpak
flatpak run com.wingdrive.desktop
```

O sandbox enxerga menos montagens que uma instalação nativa, então o `.deb`, o AppImage e os pacotes do AUR continuam sendo as instalações recomendadas no Linux.

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

Baixe o instalador NSIS (`.exe`) ou o `.msi` para x64. O SmartScreen pode exibir um alerta em instaladores sem assinatura Authenticode. Confira o arquivo com `SHA256SUMS`:

```powershell
Get-FileHash .\WingDrive-*-x64-setup.exe -Algorithm SHA256
```

## Android (beta)

Baixe o APK ARM64 na [página de download](/pt/download/) e compare o resumo SHA-256 com o arquivo `SHA256SUMS`. Abra o APK no celular para instalar. O Android pode pedir autorização para instalar pelo app que abriu o arquivo.

## Atualizações

O AppImage e os instaladores do Windows se atualizam sozinhos. Em **Configurações → Geral → Atualizações** você vê o canal, procura atualizações quando quiser e instala com uma reinicialização; o WingDrive também pode procurar uma vez depois de abrir e avisar. Toda atualização é assinada com a chave de atualização do WingDrive e conferida antes de instalar.

Os pacotes do AUR e o `.deb` se atualizam pelo gerenciador de pacotes (`yay`, `pacman` ou `apt`), então o WingDrive nunca sobrescreve arquivos que pertencem ao gerenciador.

A atualização automática começa na 3.4.0. Se você usa um AppImage ou instalador do Windows mais antigo, baixe a 3.4.0 à mão uma vez.

## macOS (prévia)

Baixe o `WingDrive-*-aarch64.dmg` para Macs com Apple silicon, abra o arquivo e arraste o WingDrive para Aplicativos. O app tem assinatura ad hoc e ainda não é notarizado, então remova a quarentena uma vez antes de abrir:

```bash
xattr -dr com.apple.quarantine /Applications/WingDrive.app
```

O app do macOS ainda não se atualiza sozinho: baixe cada novo `.dmg` na [página de download](/pt/download/). Versões notarizadas estão planejadas para o marco M3. Macs com Intel precisam compilar a partir do código-fonte (abaixo).

## Servidor sem interface gráfica

Execute o daemon com sua interface web em um NAS, servidor doméstico ou VPS:

```bash
docker run -d --name wingdrive \
  -p 8080:8080 -p 7373:7373 \
  -v wingdrive-data:/data \
  -e WING_AUTH=admin:change-me \
  ghcr.io/bernardopg/wingdrive/server:latest
```

Sem Docker, use o arquivo `wing-server-*-linux-x86_64` (`.tar.gz`, `.tar.xz` ou `.zip`): ele traz o `wing-server`, a CLI `wing` e as bibliotecas de mídia que eles carregam. Consulte [Auto-hospedagem](/pt/docs/self-hosting/) para o arquivo, Compose, systemd e proxy reverso.

No Windows, o `wing-*-windows-x64.zip` traz o `wing.exe` e o `wing-daemon.exe` para scripts e servidores sem o app desktop.

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
