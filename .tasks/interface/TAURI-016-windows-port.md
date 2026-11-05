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
- [x] `scripts/release/windows_smoke.ps1` installs, runs and uninstalls a bundle
- [x] Build `.exe` and `.msi` with the workflow's command
- [ ] Run the Windows jobs on GitHub and fix what only fails on the runner
- [ ] Add the Windows jobs to the required checks in `.github/rulesets/main.json`

## Acceptance Criteria

- [x] `cargo clippy --workspace -- -D warnings` is clean on Windows
- [x] The CI Rust gates pass on Windows: 459 `wing-core` tests, 20 sync tests, 27 desktop tests, 18 CLI tests
- [x] `just ci-local --all` passes on Windows, with Smart App Control off
- [x] Typecheck and the 33 CI frontend test files pass on Windows
- [x] The desktop app opens through `just dev-desktop` and loads a library
- [x] Location, indexing, search and volume listing work through the CLI
- [x] `.exe` and `.msi` install, run and uninstall through `windows_smoke.ps1`, locally, on 2026-11-04
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

- The NSIS uninstaller leaves its folder in `HKCU\Software\WingDrive contributors\WingDrive`,
  and the MSI's registry search then overrides `INSTALLDIR`. A user who moves from the `.exe`
  to the `.msi` lands in the old folder. The smoke test clears the key first.
- Under `shell: bash` and `just`, `perl` is Git's MSYS perl, which cannot run OpenSSL's
  `Configure`. `OPENSSL_SRC_PERL` points at Strawberry Perl.

Not specific to Windows:

- `wing` and `wing-daemon` evaluated `unwrap_or(default_data_dir()?)` eagerly, so they
  created an empty production data directory even with `--data-dir`.
- The sync loop decides to backfill from a state it read before several awaits. On the
  slower Windows runner, `sync_metrics_test` failed twice because the test harness set the
  device Ready in that window and `start_backfill` overwrote it with Backfilling 16 ms
  later. `start_backfill` now skips when the state already moved to Ready, catching up or
  paused, and `test_backfill_skipped_once_device_is_ready` covers it.

Smart App Control blocked freshly built unsigned binaries until it was turned off on the
development machine. `docs/core/windows-development.mdx` shows how to recognize it. It may
also block unsigned installers on users' machines (REL-001).

Traps in the runner setup, handled but not yet proven on GitHub:

- The image ships LLVM 20 and MinGW on the PATH. `setup-system` removes them.
- Windows PowerShell's `>>` writes `GITHUB_ENV` as UTF-16, so those steps use `pwsh`.
- Runners check out with `core.autocrlf=true`. `.gitattributes` forces LF.

## Still open

Done on 2026-11-04, before the first Windows release. Checked by unit tests and `clippy`
for `x86_64-pc-windows-msvc` from Linux, not yet by hand on Windows:

- [x] System file clipboard on Windows. `file-clipboard-windows` writes and reads the
  `CF_HDROP` list and `Preferred DropEffect`, so copy, cut and paste work with Explorer.
  The encoding is tested on every platform; the Win32 calls only on the Windows runner.
- [x] `WINGDRIVE_INSTANCE` isolates the app on Windows. The single-instance plugin uses
  `dbus_id` only on Linux and names its mutex after the bundle identifier, so a named
  instance used to hand its launch to the everyday WingDrive and exit. A named instance
  now gets its own identifier, which also gives it its own WebView2 profile.
- [x] `windows_smoke.ps1` waits for a visible "WingDrive" window owned by the app. The
  main window starts hidden and the frontend shows it once loaded, so this proves the
  WebView rendered. `MainWindowHandle` also accepted the plugin's hidden
  `com.wingdrive.desktop-siw` window.
- [x] Desktop notifications from #332 build on Windows: the Notify and portal builders
  serve only the Linux sender, so `notifications` allows dead code off Linux. The toast
  logo `icons/128x128.png` is now a bundle resource; without it the toast had no logo.
- [x] "Retry as administrator" is offered on Linux only. It used to appear on Windows and
  macOS and then fail with "only available on Linux".

Still open:

- [ ] Copy in WingDrive and paste in Explorer, and the other way round, on a real machine (TAURI-012)
- [ ] "Administrator retries" on Windows through UAC. It needs its own design: there is no
  coreutils, and an elevated `cmd` or PowerShell command line must not be built from paths.
- [ ] Authenticode signing for the installers (add to REL-001)
- [ ] Runtime checks not done yet: daemon service through `schtasks`, "Abrir com" (CORE-014), the TAURI-006 checklist (TAURI-012)
- [ ] The README still marks Windows as planned. Change the badge after the first published release.

## Handoff

The runtime fixes merged in #330 and the CI and installers in #331. Installers for any branch
come from `gh workflow run windows-bundle.yml --ref <branch>`, in the run's artifacts.

To close this task: publish a `v*` tag with the Windows jobs green, then add them to the
required checks. Everything needed to resume from Linux is in this file and in
`docs/core/windows-development.mdx`.
