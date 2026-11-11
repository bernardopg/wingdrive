---
title: Installation
description: Install WingDrive on Linux, Windows, a server, or build it from source.
order: 2
group: Start
---

Get prebuilt packages from the [download page](/download/) or from [GitHub Releases](https://github.com/bernardopg/wingdrive/releases/latest). Every release ships `SHA256SUMS` files to verify your download.

## Linux

Linux (x86_64) is the supported platform.

### AppImage

Runs on any distribution without installing anything system-wide:

```bash
chmod +x WingDrive-*-x86_64.AppImage
./WingDrive-*-x86_64.AppImage
```

### Debian, Ubuntu and derivatives

```bash
sudo apt install ./WingDrive-*-amd64.deb
```

### Arch Linux and derivatives (AUR)

Two packages install the same release AppImage, and they conflict with each other:

- `wingdrive-bin` follows stable releases only.
- `wingdrive-nightly` follows every release, alpha, beta and rc included. A stable release updates it too, so it is never behind `wingdrive-bin`.

```bash
yay -S wingdrive-bin        # or: yay -S wingdrive-nightly
```

Without an AUR helper:

```bash
git clone https://aur.archlinux.org/wingdrive-bin.git
cd wingdrive-bin
makepkg -si
```

## Windows 10 and 11 (beta)

Download the NSIS installer (`.exe`) or the `.msi` for x64. The installers are not signed yet, so SmartScreen warns on first run: choose **More info**, then **Run anyway**. Check the file against `SHA256SUMS-windows`:

```powershell
Get-FileHash .\WingDrive-*-x64-setup.exe -Algorithm SHA256
```

## macOS

Prebuilt, signed builds are planned for milestone M3. Until then, build from source (below).

## Headless server

Run the daemon with its web interface on a NAS, home server or VPS:

```bash
docker run -d --name wingdrive \
  -p 8080:8080 -p 7373:7373 \
  -v wingdrive-data:/data \
  -e WING_AUTH=admin:change-me \
  ghcr.io/bernardopg/wingdrive/server:latest
```

See [Self-hosting](/docs/self-hosting/) for Compose, systemd and reverse proxy setups.

## Build from source

You need the Rust toolchain pinned in `rust-toolchain.toml` (installed by [rustup](https://rustup.rs/)), [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just) and Python 3.9+ (for adapters).

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # native dependencies + cargo config
just dev-desktop  # desktop app; starts the daemon for you
```

Other useful recipes:

```bash
just dev-daemon   # daemon only, with ffmpeg and HEIF support
just dev-server   # headless server + web UI
just cli status   # run the wing CLI from source
just test         # run the workspace tests
```

## Verify a download

```bash
sha256sum -c SHA256SUMS --ignore-missing
```
