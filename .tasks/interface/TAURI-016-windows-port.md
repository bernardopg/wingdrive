---
id: TAURI-016
title: Windows port baseline, CI and installers
status: In Progress
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M3
sprint: S05
tags: [tauri, windows, ci, release, platform]
related_tasks: [TAURI-012, CORE-014, REL-001, TAURI-010]
last_updated: 2026-11-04
---

## Description

Make WingDrive build, test and ship on Windows, so the TAURI-012 regression matrix
has something to run against. Started on 2026-11-04 on a Windows 11 machine set to
Portuguese. How to set up and work on Windows is in
`docs/core/windows-development.mdx`.

## Implementation Steps

- [x] Reproducible toolchain: `scripts/setup.ps1`, `.gitattributes`, `scripts/dev-isolated.ps1`, `just dev-desktop`
- [x] Fix what breaks at runtime and in tests on Windows (see Findings)
- [x] Windows jobs in CI and in the Release workflow
- [x] `scripts/release/windows_smoke.ps1` installs, runs and uninstalls a bundle (written, first run is on GitHub)
- [ ] Build `.exe` and `.msi` through `windows-bundle.yml`. The local build stops on Smart App Control
- [ ] Run the Windows jobs on GitHub and fix what only fails on the runner
- [ ] Add the Windows jobs to the required checks in `.github/rulesets/main.json`

## Acceptance Criteria

- [x] `cargo clippy --workspace -- -D warnings` is clean on Windows
- [x] The CI Rust gates pass on Windows: 459 `wing-core` tests, 20 sync tests, 27 desktop tests, 18 CLI tests
      (`task-validator` could not run locally because Smart App Control blocked its build script; CI covers it)
- [x] Typecheck and the 33 CI frontend test files pass on Windows
- [x] The desktop app opens through `just dev-desktop` and loads a library
- [x] Location, indexing, search and volume listing work through the CLI
- [ ] `.exe` and `.msi` install, run and uninstall through `windows_smoke.ps1`
- [ ] `core-windows` and `frontend-windows` pass on GitHub
- [ ] A `v*` tag publishes the `.exe`, the `.msi` and `SHA256SUMS-windows`

## Findings

Bugs that only appear on Windows, all fixed on this task's branch:

- `is_process_running` parsed `tasklist`, whose "no tasks" message wraps onto two
  lines in Portuguese, so a dead process looked alive and libraries stayed locked
  for up to an hour. It now calls `OpenProcess` and `GetExitCodeProcess`.
- `terminate_daemon` killed the daemon outright, so it never closed its libraries.
  It now sends the RPC shutdown request that `wing stop` uses.
- `KeyManager` copied the secrets database while redb held its lock, which fails on
  Windows with os error 33 during the v2 upgrade.
- `local_path` turned `file:///C:/x` into the relative path `/C:/x`, so opening
  files by URI failed.
- `cmd /c ver` parsing searched for `Version`, so the OS version was empty in
  localized Windows. `parse_windows_ver_output` reads the token inside the brackets.
- Search, media and tag queries, `entry.rs` and `path_resolver` joined stored paths
  with `/`, which produced `C:\dir\sub/file`. `join_entry_path` keeps the stored
  path's separator, so cloud paths keep `/`.
- `dev-with-daemon.ts` ran `target\debug\wing-daemon.exe`, which `tauri-build` then
  failed to overwrite with os error 32. It now runs a per-instance copy.
- `tauri.windows.conf.json` requested the macOS `app` bundle and no `msi`.
- "Start at login" returned an error. It now writes the current user's Run key.

Not specific to Windows:

- `wing` and `wing-daemon` evaluated `unwrap_or(default_data_dir()?)` eagerly, so they
  created an empty production data directory even with `--data-dir`.

The development machine has Smart App Control on, which blocks some freshly built
unsigned binaries at random. `docs/core/windows-development.mdx` shows how to recognize
and get past it. It may also block unsigned installers on users' machines (REL-001).

Traps in the runner setup, handled but not yet proven on GitHub:

- The image ships LLVM 20 and MinGW on the PATH. `setup-system` removes them.
- Windows PowerShell's `>>` writes `GITHUB_ENV` as UTF-16, so those steps use `pwsh`.
- Runners check out with `core.autocrlf=true`. `.gitattributes` forces LF.

## Still open

Not started, and not needed for the baseline:

- [ ] System file clipboard on Windows (`CF_HDROP`). TAURI-010 covered Linux only.
- [ ] "Administrator retries" on Windows (`run_as_admin`, UAC instead of `pkexec`)
- [ ] Authenticode signing for the installers (add to REL-001)
- [ ] Runtime checks not done yet: daemon service through `schtasks`, "Abrir com" (CORE-014), the TAURI-006 checklist (TAURI-012)
- [ ] The README still marks Windows as planned. Change the badge after the first published release.

## Handoff

Work is split between two branches, the second stacked on the first:

1. `chore/windows-dev-environment`: fixes, tooling and documentation.
2. `ci/windows-release`: workflows and installers. It opens as a draft, because
   `auto-merge.yml` would merge it on the required checks alone, and the Windows jobs
   are not required yet.

Mark the second one ready once `Rust Workspace (Windows)`, `TypeScript (Windows)` and
`Verify Windows x86_64 installers` pass. The installers are in that last run's artifacts.
Everything needed to resume from Linux is in this file and in
`docs/core/windows-development.mdx`.
