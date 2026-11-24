---
id: ADAPT-001
title: "Registry repository and signed index format"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, registry, security]
related_tasks: [ADAPT-004, ADAPT-007]
last_updated: 2026-11-24
---

## Description

Create the official registry `<org>/wingdrive-adapters` and the format of its index. `<org>` is the GitHub organization the project moves to (decision pending: `wingdrive` is taken by an inactive personal account). Each
entry points at an adapter repository and pins what gets installed, so an install is reproducible
and a later push to the adapter repository changes nothing until the registry moves the pin.

## Implementation Steps

- [ ] Entry schema (JSON Schema in the registry): `id`, `name`, `summary` (at most two lines, 140 characters), `description`, `icon`, `repository`, `commit`, `version`, `archive_sha256`, `author`, `license`, `platforms`, `runtime` (`python3` script or compiled `binary`), `artifacts` (one archive and SHA-256 per target for compiled adapters), `permissions`, `trust` (`official`, `verified`, `community`), `categories`, `homepage`
- [ ] One file per adapter in `adapters/<id>.json`; the bots build `index.json` from them
- [ ] Archive built from the pinned commit (`git archive`) and its SHA-256 recorded at publish time
- [ ] Index signed with an Ed25519 key in minisign format. The private key lives only in a protected GitHub environment (`registry-publish`) of the registry repository that needs the maintainer's approval for every run, with an offline backup in the author's password manager; nobody else and no other workflow can read it
- [ ] WingDrive bundles two public keys, the active one and a standby one, so a lost or leaked key is replaced by a registry release plus an app release without breaking installed apps
- [ ] Index published as a release asset and on wingdrive.org (`/adapters/index.json` and `.minisig`)
- [ ] Support for extra registries by URL with their own public key, off by default
- [ ] Key rotation and revocation procedure written down: promote the standby key, generate a new standby, ship it in the next app release, and add a revocation list to the index for compromised adapter versions

## Acceptance Criteria

- [ ] Schema and an example entry merged in the registry repository
- [ ] The publish workflow produces `index.json` and a signature that `minisign -V` accepts
- [ ] A tampered index or archive fails verification in a unit test of the daemon (ADAPT-004)
- [ ] Docs in `docs/core/adapter-store.mdx` describe the format, trust levels and signing
