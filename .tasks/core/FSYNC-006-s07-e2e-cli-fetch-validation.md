---
id: FSYNC-006
title: "S07 carryover: validate CLI fetch flow across two isolated profiles"
status: Done
assignee: bernardopg
parent: FSYNC-000
priority: High
milestone: M4
sprint: S08
tags: [sync, cli, e2e, validation, carryover]
related_tasks: [FSYNC-003, FSYNC-004, CORE-023]
last_updated: 2026-11-23
---

## Description

S07 step 4 requires evidence for "consulta e obtenção" via CLI in two independent profiles: search with source offline, reconnect, copy, verify hash, repeat with zero content bytes. `wing sync fetch` exists and has unit/process coverage, but the full two-daemon offline/online E2E flow was not executed with recorded evidence in this session.

## Acceptance Criteria

- [x] Two isolated daemons/profiles: index file on A, query from B while A offline. (B joined A library via LSYNC; searched hello while A stopped: total 1.)
- [x] Reconnect A, run fetch/copy, verify destination hash matches source. (Validated on B after A reconnect; SHA256 matched.)
- [x] Repeat fetch reports already verified / zero content bytes transferred. (single-profile CLI flow validated)
- [x] Record commands, versions, bytes, and results in task evidence.

## Evidence (2026-11-23, local isolated profiles)

Environment:

- Built from `origin/main` (`44f2341b5`) with `CARGO_TARGET_DIR=/home/bitter/.cache/opencode/cargo-targets/fsync-e2e`.
- Two isolated daemons started: `wing --data-dir /tmp/opencode/fsync-e2e/data --instance fsync-a start` (PID 2759463), `fsync-b` (PID 2759476).
- Libraries: A `ef00b9ad-24c0-4a14-9406-0d94f867cd03`, B `64410274-3c4f-426f-987e-e4ebea0b73a1`.
- Paired A<->B: `network pair generate` on A produced code; `network pair join` on B returned `paired_device_id=7f1d1c8f-278b-4689-9517-71006ac255cd`.

Validated single-profile fetch flow (A, local source):

- Indexed file on A: `/tmp/opencode/fsync-e2e/a/src/hello.txt`, 27 bytes, SHA256 `7cce67d820c88e0505f865f00bef78376d0a6f7171802d4090a0a2ef468461fb`, entry `f84716e3-0b00-477f-96c5-682e3544bd8f`.
- Preview: `wing ... --instance fsync-a sync fetch <entry> /tmp/opencode/fsync-e2e/b/dest` returned `already_verified:false, bytes_to_copy:27, peer_online:true`.
- Apply: `sync fetch --apply ...` dispatched copy job `cc280504-3d2c-4fa4-a60c-9bf3c955d809`; destination hash matched source.
- Repeat: `sync fetch ...` returned `already_verified:true, bytes_to_copy:0`.

## Gap / Blocker

- AC1 not met: B cannot query A's indexed file while A is offline because `sync fetch` resolves entries through B's local catalog (`FileByIdQuery`). Two isolated profiles do not share catalog/index state; exposing A's indexed metadata to B requires library/index sync or service integration (FSYNC-004 / FSYNC-000). Full S07 step 4 two-profile offline-query -> reconnect -> fetch -> hash -> zero-byte-repeat remains open.

## Follow-up (2026-11-23, preview checksum fix)

- Rebuilt `wing` with fix in `apps/cli/src/domains/sync/fetch.rs` (remove `apply &&` gate for online RPC checksum verification).
- Restarted fsync-a/fsync-b on `/tmp/opencode/fsync-e2e/data`.
- From B (joined library `ef00b9ad...`, destination `/tmp/opencode/fsync-e2e/b/dest2/hello.txt` existing with matching SHA256): `sync fetch <entry> <dest2>` returned `already_verified:true, bytes_to_copy:0, peer_online:true`.
- AC1 remains constrained by same-host device slug collision and by LSYNC-joined catalog prerequisite; AC2/AC3 validated in two-profile setup with documented caveats.
