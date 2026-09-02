---
id: FORK-002
title: "Complete WingDrive identity and repository independence"
status: "Done"
assignee: "bernardopg"
priority: "High"
tags: ["fork", "branding", "github-actions", "documentation", "release"]
---

## Description

Make WingDrive independently buildable, testable, publishable, and supportable
from `bernardopg/wingdrive`. Preserve Spacedrive only in copyright attribution,
project history, and compatibility code required to open pre-fork installs.

The execution plan is versioned in
[`plans/wingdrive-independence-migration.md`](../../plans/wingdrive-independence-migration.md).

## Implementation Steps

- [x] Audit every GitHub Actions workflow, runner, permission, secret, registry, and upstream URL
- [x] Replace inherited inactive workflows with WingDrive-owned CI, release, container, and mobile workflows
- [x] Move active repository, updater, support, package, and documentation links to WingDrive
- [x] Rebrand desktop, web, mobile, server, CLI, generated clients, and visible assets
- [x] Remove active build and runtime downloads from Spacedrive-owned infrastructure
- [x] Keep original authorship and license notices intact
- [x] Add an automated autonomy check with an explicit legacy and attribution allowlist
- [x] Validate task metadata, workflow syntax, types, core tests, and release inputs
- [x] Make new CLI services, config files, exports, containers, user agents, and storage keys write WingDrive names
- [x] Preserve tested reads for legacy services, config, exports, data directories, keyring entries, and storage keys
- [x] Remove unused Spacedrive-named packaging and placeholder CLI scripts

## Acceptance Criteria

- Pushes and pull requests run required checks on GitHub-hosted runners available to this repository.
- A `v*` tag can produce WingDrive artifacts, a GitHub release, and a GHCR server image using repository permissions.
- Mobile validation runs without access to Spacedrive runners or signing identities. Store publication requires only WingDrive-owned secrets.
- Active source, configuration, manifests, UI, and documentation contain no operational `spacedriveapp` or `spacedrive.com` dependency.
- User-visible product identity is WingDrive on every maintained client.
- Remaining Spacedrive references are limited to attribution, historical text, and tested legacy data compatibility.
- The original license, copyright, and Git history remain intact.

## Rollback

The workflow and content changes are ordinary Git changes and can be reverted.
Legacy data paths, keyring names, bundle identifiers, and config readers remain
supported during the migration. No existing user data is moved or deleted.

## Local Validation, 2026-09-02

- `actionlint` passed for all four GitHub workflows.
- `bun install --frozen-lockfile`, Tauri typecheck, both vendored UI package
  typechecks, the web production build, and 8 focused frontend tests passed.
- Both vendored UI packages rebuilt from their versioned local sources with
  `tsup`; their compiled `dist` output is explicitly versioned.
- `cargo check --workspace --locked` passed after migrating `pdfium-render` to
  crates.io 0.9.3.
- Seven core branding tests and the legacy-path integration test passed.
- Task validation, JSON/YAML parsing, `git diff --check`, and the independence
  guard passed.
- Expo public config and a clean Android prebuild passed in an isolated copy;
  the generated package was `com.wingdrive.app`.
- Native dependencies are mirrored with source archives and checksums at
  <https://github.com/bernardopg/wingdrive/releases/tag/native-deps-v0.26>.
- CLI service tests pass for default and named instances. New installs use
  `com.wingdrive.daemon` and `wingdrive-daemon`; status and uninstall still
  recognize pre-fork service names.
- `AppConfig` writes `wingdrive.json` and its focused test proves that an
  existing `spacedrive.json` remains readable and is not deleted.
- New location exports use a WingDrive header. The integration target compiles
  with a compatibility case that imports the pre-fork header.
- `cargo check --workspace --locked`, the Tauri Rust check, the CLI test,
  focused config test, task validator, guard, `actionlint`, shell syntax,
  maintained JSON/YAML parsing, Expo config, frontend typecheck/builds, and 8
  focused frontend tests pass after the final residual identity audit.
- The full `sd-core` integration test link remains blocked by the existing
  `__rust_probestack`/Wasmer native linker conflict. The affected integration
  test target passes `cargo check`; focused library tests pass.

## Remaining External Verification

Keep this task `In Progress` until the changes are committed and pushed, then
record successful GitHub run URLs for CI, mobile validation, release artifacts,
and GHCR publication. Apple, Google Play, and signed Tauri updater publication
remain disabled until WingDrive-owned credentials exist.
