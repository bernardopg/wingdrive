---
title: Installation
description: Install WingDrive on Linux, Windows, macOS, Android, a server, or build it from source.
order: 2
group: Start
---

Get prebuilt packages from the [download page](/download/) or from [GitHub Releases](https://github.com/bernardopg/wingdrive/releases/latest). Every release ships one `SHA256SUMS` file to verify any download.

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

### Flatpak

The release also ships a Flatpak bundle on the GNOME 51 runtime:

```bash
flatpak install --user WingDrive-*-x86_64.flatpak
flatpak run com.wingdrive.desktop
```

The sandbox sees fewer mounts than a native install, so the `.deb`, the AppImage and the AUR packages remain the recommended Linux installs.

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

Download the NSIS installer (`.exe`) or the `.msi` for x64. SmartScreen may warn on builds without Authenticode signing. Check the file against `SHA256SUMS`:

```powershell
Get-FileHash .\WingDrive-*-x64-setup.exe -Algorithm SHA256
```

## Android (beta)

Download the ARM64 APK from the [download page](/download/) and compare its SHA-256 digest with `SHA256SUMS`. Open the APK on your phone to install it. Android may ask you to allow installation from the app that opened the file.

## Updates

The AppImage and the Windows installers update themselves. In **Settings → General → Updates** you see the channel, check on demand and install with a restart; WingDrive can also check once after it starts and notify you. Every update is signed with the WingDrive updater key and checked before it installs.

The AUR packages and the `.deb` update through your package manager (`yay`, `pacman` or `apt`), so WingDrive never writes over files the package manager owns.

Self-updating starts with 3.4.0. If you run an older AppImage or Windows installer, download 3.4.0 by hand once.

## macOS (preview)

Download `WingDrive-*-aarch64.dmg` for Macs with Apple silicon, open it and drag WingDrive to Applications. The app is signed ad hoc and not notarized yet, so clear the quarantine once before the first launch:

```bash
xattr -dr com.apple.quarantine /Applications/WingDrive.app
```

The macOS app does not update itself yet: download each new `.dmg` from the [download page](/download/). Notarized builds are planned for milestone M3. Intel Macs need a build from source (below).

## Headless server

Run the daemon with its web interface on a NAS, home server or VPS:

```bash
curl -O https://raw.githubusercontent.com/bernardopg/wingdrive/main/apps/server/docker-compose.yml
WING_AUTH='me:a-long-password' docker compose up -d
```

Without Docker, use the `wing-server-*-linux-x86_64` archive (`.tar.gz`, `.tar.xz` or `.zip`): it holds `wing-server`, the `wing` CLI and the media libraries they load. See [Self-hosting](/docs/self-hosting/) for the archive, Compose, systemd and reverse proxy setups.

On Windows, `wing-*-windows-x64.zip` holds `wing.exe` and `wing-daemon.exe` for scripts and servers without the desktop app.

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
