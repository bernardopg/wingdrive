<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="100" height="100" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>One file manager for all your devices and clouds.</strong><br/>
  <span>Local-first · AI-ready · Peer-to-peer · macOS · Windows · Linux · iOS · Android</span>
</p>

<p align="center">
  <a href="https://github.com/bernardopg/wingdrive/releases">
    <img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?include_prereleases&label=Version&color=7c3aed" alt="Version"/>
  </a>
  <img src="https://img.shields.io/static/v1?label=License&message=Apache-2.0&color=000" alt="License"/>
  <img src="https://img.shields.io/static/v1?label=Core&message=Rust&color=DEA584" alt="Core"/>
  <img src="https://img.shields.io/static/v1?label=Platforms&message=macOS%20%7C%20Windows%20%7C%20Linux&color=36A3FF" alt="Platforms"/>
</p>

---

<div align="center">

# Your files, everywhere. Your data, yours.

WingDrive unifies your data across every device and cloud — without centralized lock-in. Files stay where they are. WingDrive indexes them, makes them searchable, and syncs them peer-to-peer between your own machines.

</div>

---

## ✨ What is WingDrive?

A cross-device data platform. Index files, emails, notes, and external sources. Search everything. Sync via P2P.

- 🔑 **Content identity** — BLAKE3 hash on every file. Same file on two devices = same hash. Deduplication across all your machines.
- 🖥️ **Cross-device** — see all your files across all devices in one place. Offline devices stay in the index.
- 🔄 **P2P sync** — direct device-to-device via Iroh/QUIC. No servers, no cloud, no single point of failure.
- ☁️ **Cloud volumes** — S3, Google Drive, Dropbox, OneDrive, Azure, GCS as first-class volumes.
- 📱 **Nine views** — grid, list, columns, media, size, recents, search, knowledge, splat. QuickPreview for video, audio, code, documents, 3D, images.
- 🏠 **Local-first** — everything runs on your machine. No data leaves your device unless you choose to sync.

---

## 🚀 What's New in 2.0.0-alpha16

Released October 15, 2026.

<details>
<summary><strong>📁 Explorer</strong></summary>
Folders show real content size · hard-linked files count once · open folder in new window (Ctrl+N / Cmd+N) · fixed rename/move in non-indexed folders
</details>

<details>
<summary><strong>🖥️ Desktop</strong></summary>
Tray menu follows interface language · About/Wingbot links open in system browser · remaining strings translated
</details>

<details>
<summary><strong>💾 Devices & Storage</strong></summary>
Unplugged drives marked offline instead of staying online
</details>

<details>
<summary><strong>🛡️ Reliability</strong></summary>
Fixed shutdown in library watcher debounce · migration repairs missing files in descendant queries
</details>

<details>
<summary><strong>📱 Mobile</strong></summary>
English and Brazilian Portuguese · delete library · pinned locations in Browse · Tailscale labels
</details>

> [Full changelog →](./docs/readme/changelog-alpha16.md)

---

## 🖼️ See It in Action

<div align="center">
  <img src="docs/public/WingColumnView.webp" alt="Column view" width="300"/>
  <img src="docs/public/WingGridView.webp" alt="Grid view" width="300"/>
  <img src="docs/public/WingListView.webp" alt="List view" width="300"/>
  <img src="docs/public/WingMediaView.webp" alt="Media view" width="300"/>
  <img src="docs/public/WingSizeView.webp" alt="Size view" width="300"/>
</div>

---

## 🎯 Why WingDrive?

| Instead of | WingDrive gives you |
|------------|---------------------|
| Files scattered across devices and clouds | One unified view of everything |
| Uploading to the cloud for search | Local indexing on your machine |
| Vendor lock-in | Open source, Apache-2.0, no lock-in |
| Black-box AI processing | Structured APIs with permissions and screening |
| Central sync servers | Peer-to-peer sync between your own devices |

---

## 📦 Platforms

| Platform | Status |
|----------|--------|
| macOS | ✅ Supported |
| Linux | ✅ Supported (AppImage, DEB) |
| Windows | 🔜 Coming in alpha17 |
| iOS | 🔜 Coming soon |
| Android | 🔜 Coming soon |

---

## 🛠️ Installation

> Full details → [Installation Guide](./docs/readme/installation.md)

**Prebuilt release (recommended):**
[GitHub Releases](https://github.com/bernardopg/wingdrive/releases) — AppImage, Debian, Docker.

**Arch-based (AUR):**
<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/">
    <img src="https://img.shields.io/badge/AUR-wingdrive--bin-0090FF?logo=arch-linux&logoColor=white" alt="AUR wingdrive-bin"/>
  </a>
</p>

```bash
yay -S wingdrive-bin
```

**Build from source:**
```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive
just setup
just dev-desktop
```

---

## 📚 Architecture

> Full reference → [Architecture](./docs/readme/architecture.md)

Built on four principles: **VDFS**, **Content Identity**, **Transactional Actions**, **Leaderless Sync**.  
Stack: Rust · Tokio · SQLite · Iroh/QUIC · BLAKE3 · LanceDB · OpenDAL · Tauri 2 · React 19.

---

## 🔒 Privacy & Security

- **End-to-End Encryption** — P2P traffic via QUIC/TLS
- **At-Rest Encryption** — libraries encrypted on disk (SQLCipher)
- **No Telemetry** — zero tracking or analytics
- **Data Sovereignty** — you control where your data lives

Safety screening: Prompt Guard 2, trust tiers, quarantine system, content fencing.

---

## 🗺️ Roadmap

- **alpha17** — Windows desktop support
- **Future** — iOS app, Android app, server edition, Extension SDK (stable API), local AI models

[Releases →](https://github.com/bernardopg/wingdrive/releases)

---

## 🤝 Community

- **Discord** — [discord.gg/gTaF2Z44f5](https://discord.gg/gTaF2Z44f5)
- **Issues** — [report bugs / request features](https://github.com/bernardopg/wingdrive/issues/new/choose)
- **Whitepaper** — [docs/overview/whitepaper.mdx](./docs/overview/whitepaper.mdx)

---

## 📄 License

**Apache-2.0** — see [LICENSE](./LICENSE). File-kind icons in `packages/assets/icons` stay under GPL-3.0-only; see [NOTICE.md](./NOTICE.md).
