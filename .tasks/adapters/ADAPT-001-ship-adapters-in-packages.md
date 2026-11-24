---
id: ADAPT-001
title: "Ship adapters in every package and find them at run time"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, packaging, release, core]
related_tasks: [ADAPT-003]
last_updated: 2026-11-24
---

## Description

`SourceManager::sync_bundled_adapters` copies adapters from `env!("CARGO_MANIFEST_DIR")/../adapters`,
a path that exists only on the machine that built the binary, and `Engine::source_adapters_dir` looks
in `crates/adapters`, which does not exist. No package carries `adapters/`, so every install shows
"No adapters installed" and "update available" never works.

## Implementation Steps

- [ ] Resolve the bundled adapters directory at run time: next to the executable (`<bin>/../share/wingdrive/adapters` on Linux, the app resources on macOS and Windows), then `WINGDRIVE_ADAPTERS_DIR`, then the source tree for dev builds
- [ ] One function used by both the copy into the library and `source_adapters_dir`
- [ ] Package `adapters/` (without `__pycache__`) in the `.deb`, AppImage, AUR, Flatpak, macOS app, Windows installers and the server archive and Docker image
- [ ] Declare `python3` as a dependency where the package format allows it (AUR, `.deb`), and report a missing interpreter in the adapter list instead of failing the sync silently
- [ ] `release.yml` checks that each package carries `adapters/*/adapter.toml`

## Acceptance Criteria

- [ ] A daemon built in CI and installed from the AUR package lists the bundled adapters on a fresh library
- [ ] `adapters.list` reports `update_available` when a bundled adapter is newer than the installed copy
- [ ] Unit test for the directory resolution order
- [ ] Release run shows the adapter check passing for every package
