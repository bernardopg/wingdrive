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

Create the official registry `bernardopg/wingdrive-adapters` and the format of its index. Each
entry points at an adapter repository and pins what gets installed, so an install is reproducible
and a later push to the adapter repository changes nothing until the registry moves the pin.

## Implementation Steps

- [ ] Entry schema (JSON Schema in the registry): `id`, `name`, `summary` (at most two lines, 140 characters), `description`, `icon`, `repository`, `commit`, `version`, `archive_sha256`, `author`, `license`, `platforms`, `runtime` (`python3 >= 3.9`), `permissions`, `trust` (`official`, `verified`, `community`), `categories`, `homepage`
- [ ] One file per adapter in `adapters/<id>.json`; the bots build `index.json` from them
- [ ] Archive built from the pinned commit (`git archive`) and its SHA-256 recorded at publish time
- [ ] Index signed with an Ed25519 key (minisign format); the public key ships in WingDrive and the private key lives only in the registry's release secret
- [ ] Index published as a release asset and on wingdrive.org (`/adapters/index.json` and `.minisig`)
- [ ] Support for extra registries by URL with their own public key, off by default
- [ ] Key rotation procedure written down (two keys accepted during a rotation)

## Acceptance Criteria

- [ ] Schema and an example entry merged in the registry repository
- [ ] The publish workflow produces `index.json` and a signature that `minisign -V` accepts
- [ ] A tampered index or archive fails verification in a unit test of the daemon (ADAPT-004)
- [ ] Docs in `docs/core/adapter-store.mdx` describe the format, trust levels and signing
