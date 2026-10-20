---
id: SEC-010
title: Other local users could read the device key and drive the daemon
status: Done
assignee: bernardopg
parent: SEC-000
priority: High
milestone: M2
sprint: S03
tags: [security, daemon, crypto, privacy]
last_updated: 2026-10-20
---

## Description

Data-leak audit after 2.0.1.

1. `~/.wingdrive` was created 0755 and `device_key` 0644. Any local account could read the device key, which encrypts `secrets.redb` (cloud credentials), and read the library databases and logs.
2. The daemon's JSON-RPC socket on `127.0.0.1:6969` has no authentication. Any local account could list, copy or delete the user's files, subscribe to events and logs, or shut the daemon down.
3. A malformed RPC request was logged in full at ERROR level; a request carrying cloud credentials would reach `daemon.log`.

Repository history: TruffleHog found no verified secret. The 37 gitleaks findings are placeholders, public keys and the Azure storage emulator key in vendored code.

## Implementation Steps

- [x] `AppConfig::ensure_directories` sets the data directory to 0700
- [x] The device key fallback file is written 0600 and existing files are tightened on read; the secrets backup keeps 0600
- [x] On Linux the RPC server reads the peer socket owner from `/proc/net/tcp` and closes connections from other uids
- [x] Raw malformed requests are logged at DEBUG

## Acceptance Criteria

- [x] A fresh or existing data directory ends up 0700, the device key 0600
- [x] `nobody` connecting to the daemon is disconnected; the owner still gets `Pong`

## Evidence (2026-10-20)

- `data_directory_is_private`, `device_key_file_is_private`, `peer_tests::finds_the_owner_of_the_connecting_socket`.
- Disposable daemon: `Ping` as the user returned `Pong`; as `nobody` the connection closed and the daemon logged `Rejected an RPC connection from another local user`.

## Follow-up

- Fixed in a follow-up PR: the Tauri sidecar HTTP server served `/sidecar/...` without its session token, and its traversal check was lexical, so `..` segments in the variant reached any file the user can read. Segments are now validated and the token is required.
- Task files and sprint logs in the public repo name the local username and hosts (`/home/bitter`, `homesystem`).
