<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="120" height="120" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>One file manager for all your devices and clouds.</strong><br/>
	<span>Powered by a Virtual Distributed File System, complete with apps for macOS, Windows, Linux, iOS and Android</span>
</p>

<p align="center">
  <a href="https://fsl.software/">
    <img src="https://img.shields.io/static/v1?label=License&message=FSL-1.1-ALv2&color=000" />
  </a>
  <img src="https://img.shields.io/static/v1?label=Core&message=Rust&color=DEA584" />
  <img src="https://img.shields.io/static/v1?label=Status&message=community%20fork&color=7c3aed" />
</p>

<p align="center">
  <a href="#getting-started">Getting Started</a> &bull;
  <a href="./NOTICE.md">Notice &amp; License</a> &bull;
  <a href="./CONTRIBUTING.md">Contributing</a>
</p>

---

> **WingDrive is a community continuation fork of [Spacedrive](https://github.com/spacedriveapp/spacedrive).**
>
> Upstream development stalled, so this fork carries the work forward in the open.
> It preserves the full upstream history and keeps the original copyright intact.
>
> WingDrive is **not affiliated with, endorsed by, or supported by Spacedrive
> Technology Inc.** Please do not report WingDrive issues upstream.
>
> The code is **source-available under FSL-1.1-ALv2**, not OSI open source. The
> license forbids relicensing, so this fork cannot change it. It converts to
> Apache-2.0 in 2028 under the Grant of Future License. See [NOTICE.md](./NOTICE.md).

---

## What is WingDrive?

WingDrive is a cross-device data platform. Index files, emails, notes, and external sources. Search everything. Sync via P2P. Keep AI agents safe with built-in screening.

- **Content identity** — every file gets a BLAKE3 content hash. Same file on two devices produces the same hash. WingDrive tracks redundancy and deduplication across all your machines.
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

### Data Archival

WingDrive indexes external data sources via script-based adapters: Gmail, Apple Notes, Chrome bookmarks, Obsidian, Slack, GitHub, calendar events, contacts. Each source becomes a searchable repository alongside your files.

Adapters are a folder with an `adapter.toml` manifest and a sync script in any language. If it reads stdin and prints lines, it works.

**Shipped adapters:** Gmail, Apple Notes, Chrome Bookmarks, Chrome History, Safari History, Obsidian, OpenCode, Slack, macOS Contacts, macOS Calendar, GitHub.

### Spacebot

WingDrive integrates with [Spacebot](https://github.com/spacedriveapp/spacebot), an AI agent runtime maintained upstream. Spacebot runs as an optional separate process. WingDrive provides the data, permission, and execution layer. Spacebot provides the intelligence.

The Spacebot runtime is not bundled with this repository. When it is absent the desktop app renders an explicit unavailable state instead of simulating success.

Each Spacebot instance pairs with one WingDrive node as its home device. That node authenticates the agent, maintains the device graph, resolves permissions, and forwards operations to peer devices. Every device in your library can reach Spacebot through the paired node over P2P (Iroh/QUIC) without direct network access. One agent runtime serves your entire device fleet.

When Spacebot spawns a worker, that worker can target any device in the library. File reads, shell commands, and operations proxy through WingDrive to the target device. Talk to the agent from your phone while work executes on a server. Read files from a NAS, run commands on a workstation, report to a laptop — all in one task.

Every operation passes through WingDrive's permission system: which devices the agent can access, which paths are readable or writable, which operations are allowed, and which require human confirmation. The paired node resolves effective policy before forwarding. One security model, one audit surface across all devices and clouds.

### File System Intelligence

WingDrive adds intelligence to your filesystem by combining three layers:

- **File intelligence** — derivative data like OCR, transcripts, extracted metadata, thumbnails, previews, classifications, and sidecars.
- **Directory intelligence** — contextual knowledge attached to folders and subtrees ("active projects", "dormant archives", etc).
- **Access intelligence** — permissions and policy that apply across devices and clouds, routing agents through structured access instead of raw shell commands.

When an agent navigates through WingDrive, it receives the file listing, subtree context, effective permissions, and summaries. Users can explain how they organize their system. Agents can add attributed notes. Jobs generate summaries from structure and activity. The intelligence stays attached to the filesystem, not buried in temporary session memory.

### Safety Screening

When enabled, every record passes through a safety pipeline before becoming searchable:

- **Prompt Guard 2** — local classifier detects prompt injection in emails, messages, and documents before they enter the index.
- **Trust tiers** — authored content (your notes) gets balanced screening, external content (email inbox) gets strict screening.
- **Quarantine system** — flagged records excluded from AI agent queries, reviewable in desktop app.
- **Content fencing** — search results include trust metadata so agents know what's safe vs untrusted.

No other local data tool screens indexed content before exposing it to AI.

---

## Architecture

The core is built on four principles:

1. **Virtual Distributed Filesystem (VDFS)** — files and folders become first-class objects with rich metadata, independent of physical location. Every file gets a universal address (`SdPath`) that works across devices.

2. **Content Identity System** — adaptive hashing (BLAKE3 with strategic sampling for large files) creates a unique fingerprint for every piece of content. Enables deduplication, redundancy tracking, and content-based operations.

3. **Transactional Actions** — every file operation can be previewed before execution. See space savings, conflicts, and estimated time, then approve or cancel. Operations become durable jobs that survive network interruptions and device restarts.

4. **Leaderless Sync** — peer-to-peer synchronization without central coordinators. Device-specific data uses state replication. Shared metadata uses an HLC-ordered log with deterministic conflict resolution.

The implementation is a single Rust crate with CQRS/DDD architecture. Every operation (file copy, tag create, search query) is a registered action or query with type-safe input/output that auto-generates TypeScript types for the frontend.

| Component       | Technology                                   |
| --------------- | -------------------------------------------- |
| Language        | Rust                                         |
| Async runtime   | Tokio                                        |
| Database        | SQLite (SeaORM + sqlx)                       |
| P2P             | Iroh (QUIC, hole-punching, local discovery)  |
| Content hashing | BLAKE3                                       |
| Vector search   | LanceDB + FastEmbed                          |
| Cloud storage   | OpenDAL                                      |
| Cryptography    | Ed25519, X25519, ChaCha20-Poly1305, AES-GCM  |
| Media           | FFmpeg, libheif, Pdfium, Whisper             |
| Desktop         | Tauri 2                                      |
| Mobile          | React Native + Expo                          |
| Frontend        | React 19, Vite, TanStack Query, Tailwind CSS v4 |
| Design system   | [SpaceUI](https://github.com/spacedriveapp/spaceui) (upstream component library) |
| Type generation | Specta                                       |

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
│   ├── ui/                # Component library
│   └── assets/            # Icons, images, SVGs
├── crates/                # Standalone Rust crates (ffmpeg, crypto, etc.)
├── adapters/              # Script-based data source adapters
└── schemas/               # TOML data type schemas
```

---

## Getting Started

Requires [Rust](https://rustup.rs/) 1.81+, [Bun](https://bun.sh) 1.3+, [just](https://github.com/casey/just), and Python 3.9+ (for adapters).

```bash
git clone https://github.com/bernardopg/wingdrive
cd wingdrive

just setup        # bun install + native deps + cargo config
just dev-desktop  # launch the desktop app (auto-starts daemon)
just test         # run all workspace tests
```

---

## Privacy & Security

WingDrive is local-first. Your data stays on your devices.

- **End-to-End Encryption** — all P2P traffic encrypted via QUIC/TLS
- **At-Rest Encryption** — libraries can be encrypted on disk (SQLCipher)
- **No Telemetry** — zero tracking or analytics
- **Self-Hostable** — run your own relay servers
- **Data Sovereignty** — you control where your data lives

Optional cloud integration is available for backup and remote access, but it's never required. The cloud service runs unmodified core as a standard P2P device—no special privileges.

---

## Contributing

Contributions are welcome from anyone. Open an issue or a pull request.

- **[Contributing Guide](CONTRIBUTING.md)**
- **[Adapter Guide](docs/ADAPTERS.md)** — write a data source adapter
- **[SpaceUI](https://github.com/spacedriveapp/spaceui)** — upstream design system (clone alongside WingDrive to work on UI)

By opening a pull request you agree that your contribution is licensed under
FSL-1.1-ALv2, the same terms as the rest of the project.

---

## License

**FSL-1.1-ALv2** — [Functional Source License](https://fsl.software/). Copyright
2026 Spacedrive Technology Inc.

Source-available, not OSI open source. Free for personal use, internal use,
non-commercial education and non-commercial research. Offering the software as a
managed, hosted or SaaS product to third parties is not permitted.

Converts to Apache-2.0 in 2028 under the Grant of Future License.

Full details and fork attribution: [NOTICE.md](./NOTICE.md).
