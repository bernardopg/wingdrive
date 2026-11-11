---
title: Architecture
description: How the daemon, clients, library and networking fit together.
order: 11
group: Advanced
---

WingDrive follows a daemon-client design. One `wing-daemon` process owns all state. Every client is a thin front-end that speaks JSON-RPC 2.0 to it.

```text
Desktop app (Tauri + React) ┐
wing CLI                    ├─ JSON-RPC ─▶ wing-daemon ─▶ actions / queries ─▶ services ─▶ SQLite
wing-server + web UI        ┘
```

The mobile app is the exception: it embeds the same Rust core as a library instead of talking to a daemon.

## The stack

| Layer | Technology |
|---|---|
| Core | Rust, Tokio |
| Database | SQLite through SeaORM |
| Networking | Iroh over QUIC |
| Content identity | BLAKE3 |
| Cloud storage | OpenDAL |
| Desktop | Tauri 2 and React 19 |

## Inside the core

The core separates **nouns** from **verbs**:

- **Domain**: files, devices, volumes, locations, tags and addressing.
- **Operations**: every user-facing verb is an *action* (writes) or a *query* (reads), registered at compile time. The same operation serves the desktop app, the CLI and the web.
- **Jobs**: long operations persist their progress and resume after a restart.
- **Services**: long-lived subsystems such as networking, sync, the filesystem watcher and the volume monitor.

## Indexing

Adding a location starts an indexing job that walks the folder, records entries in the library and, depending on the mode, computes content identity and media data such as thumbnails. A watcher keeps the index current as files change.

Content identity uses BLAKE3. Small files are hashed whole; files over 100 KB are sampled (58 KB in total), which keeps cloud indexing fast and predictable.

## Addressing

Every file has a **WingPath**:

- **Physical**: a device and a path on it.
- **Cloud**: a service, a bucket or drive, and a path.
- **Content**: a content ID that resolves to any available copy.

This is what lets copy and move work the same between a laptop, an external drive and a bucket.

## Privacy by design

Indexing, thumbnails and search run locally. Traffic between paired devices is encrypted. Cloud credentials are encrypted at rest in the OS keyring. Nothing reports home.

## Learn more

The repository holds the full technical documentation: [docs/ on GitHub](https://github.com/bernardopg/wingdrive/tree/main/docs).
