<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="120" height="120" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>One file manager for all your devices and clouds.</strong><br/>
  <span>Local-first, AI-ready, peer-to-peer file manager for macOS, Windows, Linux, iOS and Android</span>
</p>

<p align="center">
  <a href="./LICENSE">
    <img src="https://img.shields.io/static/v1?label=License&message=Apache-2.0&color=000" />
  </a>
  <img src="https://img.shields.io/static/v1?label=Core&message=Rust&color=DEA584" />
  <img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?include_prereleases&label=Version&color=7c3aed" />
  <img src="https://img.shields.io/static/v1?label=Platforms&message=macOS%20%7C%20Windows%20%7C%20Linux&color=36A3FF" />
</p>

<p align="center">
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="#whats-new-in-alpha16">What&#8217;s New in alpha16</a> &bull;
  <a href="#platforms">Platforms</a> &bull;
  <a href="#roadmap">Roadmap</a> &bull;
  <a href="https://github.com/bernardopg/wingdrive/releases">Download</a> &bull;
  <a href="./NOTICE.md">Notice &amp; License</a> &bull;
  <a href="./CONTRIBUTING.md">Contributing</a>
</p>

---

> **WingDrive is a community continuation fork of [Spacedrive](https://github.com/spacedriveapp/spacedrive).**
>
> Upstream development stalled, so this fork carries the work forward in the open. It preserves the full upstream history and keeps the original copyright intact.
>
> WingDrive is **not affiliated with, endorsed by, or supported by Spacedrive Technology Inc.** Please do not report WingDrive issues upstream.
>
> The code is licensed under **Apache-2.0**. See [NOTICE.md](./NOTICE.md) for full details and fork attribution.

---

<div align="center">

# Your files, everywhere. Your data, yours.

WingDrive is a local-first file manager that unifies your data across every device and cloud service without centralized cloud lock-in. Files stay where they are. WingDrive indexes them, makes them searchable, and syncs them peer-to-peer between your own machines.

</div>

---

## What&#8217;s New in 2.0.0-alpha16

Released October 15, 2026. Full details in the [CHANGELOG](./CHANGELOG.md).

<details>
<summary><strong>Explorer</strong></summary>

- Folders show their real content size; optional "Calculate Folder Sizes" for non-indexed folders
- Hard-linked files count once in folder sizes and file counts
- Open a folder in a new window from the context menu or with Ctrl+N (Cmd+N on macOS)
- Renaming or moving a folder in a non-indexed folder now updates the view and reindexes without errors

</details>

<details>
<summary><strong>Desktop</strong></summary>

- Tray menu follows the interface language, including hidden autostart launches
- About page and Wingbot links open in the system browser
- Remaining hard-coded interface strings are translated

</details>

<details>
<summary><strong>Devices and storage</strong></summary>

- An unplugged drive is marked offline instead of staying online

</details>

<details>
<summary><strong>Reliability</strong></summary>

- Shutting down inside the library watcher&#8217;s debounce no longer reopens the library or leaves lock files behind
- Migration repairs folders and files missing from descendant queries

</details>

<details>
<summary><strong>Mobile</strong></summary>

- Available in English and Brazilian Portuguese
- Delete a library, select a new library after creation, pinned locations appear in Browse, Tailscale connections are labelled

</details>

---

## What is WingDrive?

WingDrive is a cross-device data platform. Index files, emails, notes, and external sources. Search everything. Sync via P2P. Keep AI agents safe with built-in screening.

- **Content identity** — every file gets a BLAKE3 content hash. The same file on two devices produces the same hash. WingDrive tracks redundancy and deduplication across all your machines.
- **Cross-device** — see all your files across all your devices in one place. Files on disconnected devices stay in the index and appear as offline.
- **P2P sync** — devices connect directly via Iroh/QUIC. No servers, no cloud, no single point of failure. Metadata syncs between devices. Files stay where they are.
- **Cloud volumes** — index S3, Google Drive, Dropbox, OneDrive, Azure, and GCS as first-class volumes alongside local storage.
- **Nine views** — grid, list, columns, media, size, recents, search, knowledge, and splat. QuickPreview for video, audio, code, documents, 3D, and images.
- **Local-first** — everything runs on your machine. No data leaves your device unless you choose to sync between your own devices.

### Is this a replacement for Finder or Explorer?

No. WingDrive sits above your OS file manager and adds capabilities Finder/Explorer lack:

- **Portal across everything** — search and browse files across local disks, external drives, NAS, cloud storage, and archived data sources from one interface.
- **Operating surface for files** — content identity, sidecars, derivative artifacts, rich metadata, sync, and cross-device awareness built into the core model.
- **Embeddable and shareable** — run it as a desktop app, headless server, hosted file service, or embed the interface and APIs into other products.
- **AI-ready by design** — indexing and analysis pipelines prepare data ahead of time instead of giving agents raw shell access.
- **Safer access model** — route AI and automation through structured APIs, permissions, and processing layers instead of direct file operations.

You still use your OS for low-level file interactions. WingDrive adds the cross-platform, cross-device, cloud-aware, and automation-friendly layer on top.

---

## See It in Action

<div align="center">
  <img src="docs/public/WingColumnView.webp" alt="Column view file explorer" width="600"/>
  <p><em>Column view — browse your library across devices</em></p>
</div>

<div align="center" style="margin-top: 2rem;">
  <img src="docs/public/WingGridView.webp" alt="Grid view" width="300"/>
  <img src="docs/public/WingListView.webp" alt="List view" width="300"/>
  <img src="docs/public/WingMediaView.webp" alt="Media view" width="300"/>
  <img src="docs/public/WingSizeView.webp" alt="Size view" width="300"/>
  <p><em>Grid, list, media, and size views</em></p>
</div>

---

## Why WingDrive?

Computing was designed for single devices. File managers like Finder and Explorer assume your data lives on the computer in front of you. The shift to multi-device forced us into cloud ecosystems where convenience requires giving up data ownership.

AI is accelerating this trend. Cloud services offer semantic search and intelligent analysis, but only if you upload your files to their infrastructure. WingDrive builds the architecture for local AI on hardware you already own, on data that never leaves your control.

| Instead of | WingDrive gives you |
|------------|---------------------|
| Files scattered across devices and clouds | One unified view of everything |
| Uploading to the cloud for search | Local indexing that stays on your machine |
| Vendor lock-in | Open source, Apache-2.0, no lock-in |
| Black-box AI processing | Structured APIs with permissions and screening |
| Central sync servers | Peer-to-peer sync between your own devices |

---

## Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| macOS | Supported | Native desktop app |
| Linux | Supported | Native desktop app, AppImage and DEB |
| Windows | Coming in alpha17 | Desktop app in progress |
| iOS | Coming soon | React Native prototype |
| Android | Coming soon | React Native prototype |

---

## Getting Started

WingDrive requires [Rust](https://rustup.rs/) 1.89+, [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just), and Python 3.9+ (for adapters).

### Option 1: Download a prebuilt release (recommended)

Get the latest build from [GitHub Releases](https://github.com/bernardopg/wingdrive/releases):

- **Linux:** AppImage or Debian package
- **macOS:** coming as a native bundle
- **Docker:** `ghcr.io/bernardopg/wingdrive/server` for headless deployments

### Option 2: Build from source

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # bun install + native deps + cargo config
just dev-desktop  # launch the desktop app (auto-starts daemon)
just test         # run all workspace tests
```

Then use the CLI:

```bash
cargo run --bin wing -- <command>
```

### First run

1. Launch the desktop app.
2. Add your first library location (a folder, external drive, or cloud volume).
3. WingDrive indexes the content and makes it searchable across all your devices.

---

## Privacy & Security

WingDrive is local-first. Your data stays on your devices.

- **End-to-End Encryption** — all P2P traffic encrypted via QUIC/TLS
- **At-Rest Encryption** — libraries can be encrypted on disk (SQLCipher)
- **No Telemetry** — zero tracking or analytics
- **Self-Hostable** — run your own relay servers
- **Data Sovereignty** — you control where your data lives

Optional cloud integration is available for backup and remote access, but it is never required. The cloud service runs unmodified core as a standard P2P device—no special privileges.

### Safety Screening

When enabled, every record passes through a safety pipeline before becoming searchable:

- **Prompt Guard 2** — local classifier detects prompt injection in emails, messages, and documents before they enter the index.
- **Trust tiers** — authored content (your notes) gets balanced screening; external content (email inbox) gets strict screening.
- **Quarantine system** — flagged records are excluded from AI agent queries and are reviewable in the desktop app.
- **Content fencing** — search results include trust metadata so agents know what is safe versus untrusted.

No other local data tool screens indexed content before exposing it to AI.

---

## Architecture

The core is built on four principles:

1. **Virtual Distributed Filesystem (VDFS)** — files and folders become first-class objects with rich metadata, independent of physical location. Every file gets a universal address (`WingPath`) that works across devices.
2. **Content Identity System** — adaptive hashing (BLAKE3 with strategic sampling for large files) creates a unique fingerprint for every piece of content. Enables deduplication, redundancy tracking, and content-based operations.
3. **Transactional Actions** — every file operation can be previewed before execution. See space savings, conflicts, and estimated time, then approve or cancel. Operations become durable jobs that survive network interruptions and device restarts.
4. **Leaderless Sync** — peer-to-peer synchronization without central coordinators. Device-specific data uses state replication. Shared metadata uses an HLC-ordered log with deterministic conflict resolution.

The implementation is a single Rust crate with CQRS/DDD architecture. Every operation (file copy, tag create, search query) is a registered action or query with type-safe input/output that auto-generates TypeScript types for the frontend.

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Async runtime | Tokio |
| Database | SQLite (SeaORM + sqlx) |
| P2P | Iroh (QUIC, hole-punching, local discovery) |
| Content hashing | BLAKE3 |
| Vector search | LanceDB + FastEmbed |
| Cloud storage | OpenDAL |
| Cryptography | Ed25519, X25519, ChaCha20-Poly1305, AES-GCM |
| Media | FFmpeg, libheif, Pdfium, Whisper |
| Desktop | Tauri 2 |
| Mobile | React Native + Expo |
| Frontend | React 19, Vite, TanStack Query, Tailwind CSS v4 |
| Design system | Vendored `@wingdrive/primitives`, `@wingdrive/tokens`, and `@wingdrive/ai` workspaces |
| Type generation | Specta |

```
wingdrive/
├── core/                  # Rust engine (CQRS/DDD)
├── apps/
│   ├── tauri/             # Desktop app (macOS, Windows, Linux)
│   ├── mobile/            # React Native (iOS, Android)
│   ├── cli/               # CLI and daemon
│   ├── server/            # Headless server
│   └── web/               # Browser client
├── packages/
│   ├── interface/         # Shared React UI
│   ├── ts-client/         # Auto-generated TypeScript client
│   ├── wingdrive-primitives/ # Component library
│   ├── wingdrive-tokens/  # Design tokens
│   ├── wingdrive-ai/      # Agent interface components
│   └── assets/            # Icons, images, SVGs
├── crates/                # Standalone Rust crates (ffmpeg, crypto, etc.)
├── adapters/              # Script-based data source adapters
└── schemas/               # TOML data type schemas
```

---

## Roadmap

- **alpha17** — Windows desktop support
- **Future releases** — iOS app, Android app, server edition for headless deployments, Extension SDK (stable API), AI integration with local models

See the [releases page](https://github.com/bernardopg/wingdrive/releases) for the public roadmap and the [task board](/.tasks/) for internal tracking.

---

## Community

- **Discord** — [discord.gg/gTaF2Z44f5](https://discord.gg/gTaF2Z44f5)
- **GitHub Issues** — [report bugs or request features](https://github.com/bernardopg/wingdrive/issues/new/choose)
- **Whitepaper** — [docs/overview/whitepaper.mdx](./docs/overview/whitepaper.mdx) (work in progress)
- **Architecture docs** — start with [core/architecture](./docs/core/architecture.mdx)

---

## Contributing

Contributions are welcome from anyone. Open an issue or a pull request.

- **[Contributing Guide](CONTRIBUTING.md)**
- **[Adapter Guide](docs/ADAPTERS.md)** — write a data source adapter
- The design-system workspaces live in this repository under `packages/wingdrive-*`.

By opening a pull request you agree that your contribution is licensed under Apache-2.0, the same terms as the rest of the project.

---

## License

**Apache-2.0** — see [LICENSE](./LICENSE). Copyright 2026 Spacedrive Technology Inc. and WingDrive contributors. File-kind icons in `packages/assets/icons` stay under GPL-3.0-only; see [NOTICE.md](./NOTICE.md).

Converts to Apache-2.0 in 2028 under the Grant of Future License.

Full details and fork attribution: [NOTICE.md](./NOTICE.md).