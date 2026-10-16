# Architecture

WingDrive is built on four principles:

1. **Virtual Distributed Filesystem (VDFS)** — files and folders are first-class objects with rich metadata, independent of physical location. Every file gets a universal address (`WingPath`) that works across devices.
2. **Content Identity System** — adaptive hashing (BLAKE3 with strategic sampling) creates a unique fingerprint for every piece of content. Enables deduplication, redundancy tracking, and content-based operations.
3. **Transactional Actions** — every file operation can be previewed before execution. Operations become durable jobs that survive network interruptions and device restarts.
4. **Leaderless Sync** — peer-to-peer synchronization without central coordinators. Device-specific data uses state replication; shared metadata uses an HLC-ordered log with deterministic conflict resolution.

## Tech stack

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
| Type generation | Specta |

## Repository structure

```
wingdrive/
├── core/                  # Rust engine (CQRS/DDD)
├── apps/
│   ├── tauri/             # Desktop app (macOS, Windows, Linux)
│   ├── mobile/            # React Native (iOS, Android)
│   ├── cli/               # CLI and daemon
│   ├── server/            # Headless server
│   └── web/               # Browser client
├── packages/              # Shared React UI, ts-client, design tokens, assets
├── crates/                # Standalone Rust crates (ffmpeg, crypto, etc.)
├── adapters/              # Script-based data source adapters
└── schemas/               # TOML data type schemas