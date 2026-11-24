# Installation Guide

WingDrive builds with the Rust toolchain pinned in `rust-toolchain.toml` (installed by [rustup](https://rustup.rs/)), [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just), and Python 3.9+ (for adapters).

## Option 1: Prebuilt release (recommended)

Get the latest build from [GitHub Releases](https://github.com/bernardopg/wingdrive/releases):

- **Linux (x86_64):** AppImage, Debian package or Flatpak bundle. The `.deb`, the AppImage and the
  AUR packages see every mount; the Flatpak sandbox sees fewer volumes.
- **Windows 10 and 11 (x64), beta:** NSIS installer (`.exe`) or `.msi`. SmartScreen may warn on
  builds without Authenticode signing.
- **macOS on Apple silicon, preview:** `.dmg`, signed ad hoc and not notarized yet. After copying
  the app to Applications, run `xattr -dr com.apple.quarantine /Applications/WingDrive.app` once.
  The macOS app does not update itself yet.
- **Android (ARM64), beta:** APK.
- **Server and command line:** `wing-server-<version>-linux-x86_64` as `.tar.gz`, `.tar.xz` or
  `.zip`, and `wing-<version>-windows-x64.zip` with `wing.exe` and `wing-daemon.exe`.
- **Docker:** `ghcr.io/bernardopg/wingdrive/server` (amd64 and arm64) for headless deployments

Every release has one `SHA256SUMS` for all its files:

```bash
sha256sum -c SHA256SUMS --ignore-missing
```

Install the Flatpak bundle with:

```bash
flatpak install --user WingDrive-<version>-x86_64.flatpak
flatpak run com.wingdrive.desktop
```

## Option 2: Arch-based systems (AUR)

<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/badge/AUR-wingdrive--bin-0090FF?logo=arch-linux&logoColor=white" alt="AUR wingdrive-bin"/>
  </a>
</p>

Two packages install the same release AppImage, and they conflict with each other:

- `wingdrive-bin` follows stable releases only.
- `wingdrive-nightly` follows every release, alpha, beta and rc included. A stable
  release updates it too, so it is never behind `wingdrive-bin`.

```bash
# With an AUR helper (yay / paru)
yay -S wingdrive-bin        # or: yay -S wingdrive-nightly

# Or manually
git clone https://aur.archlinux.org/wingdrive-bin.git
cd wingdrive-bin
makepkg -si
```

## Option 3: Build from source

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # install deps + configure cargo
just dev-desktop  # launch desktop app (auto-starts daemon)
just test         # run all workspace tests
```

Then use the CLI:

```bash
cargo run --bin wing -- <command>
```

## First run

1. Launch the desktop app.
2. Add your first library location (folder, external drive, or cloud volume).
3. WingDrive indexes the content and makes it searchable across all your devices.
