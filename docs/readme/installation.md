# Installation Guide

WingDrive requires [Rust](https://rustup.rs/) 1.89+, [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just), and Python 3.9+ (for adapters).

## Option 1: Prebuilt release (recommended)

Get the latest build from [GitHub Releases](https://github.com/bernardopg/wingdrive/releases):

- **Linux:** AppImage or Debian package
- **macOS:** native bundle coming soon
- **Docker:** `ghcr.io/bernardopg/wingdrive/server` for headless deployments

## Option 2: Arch-based systems (AUR)

<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/badge/AUR-wingdrive--bin-0090FF?logo=arch-linux&logoColor=white" alt="AUR wingdrive-bin"/>
  </a>
</p>

```bash
# With an AUR helper (yay / paru)
yay -S wingdrive-bin

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
