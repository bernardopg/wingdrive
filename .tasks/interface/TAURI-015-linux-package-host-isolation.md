---
id: TAURI-015
title: Linux package host isolation and daemon shutdown
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M2
sprint: S03
tags: [tauri, linux, aur, packaging, daemon]
last_updated: 2026-10-07
---

## Description

First launch of `wingdrive-bin` 2.0.0alpha8 on Arch showed three defects. The AUR package
extracted the AppImage and ran its `AppRun`, which put the bundled Ubuntu GTK/WebKit/glib 2.80
stack on `LD_LIBRARY_PATH`. Host tools spawned by WingDrive inherited it, so `gdbus` failed
with `undefined symbol: g_variant_builder_init_static` and Reveal fell back to opening the parent
folder. The bundled WebKit running against Arch's glibc aborted on exit with
`free(): corrupted unsorted chunks`. Closing the app also left the daemon it had started
running, while the log claimed it was shutting it down.

## Implementation Steps

- [x] `wingdrive-bin` installs only the binaries, resources and the FFmpeg 61 libraries the daemon links;
      GTK, WebKit and glib come from Arch, and `package()` fails on any unresolved library
- [x] Host tools (`gdbus`, `xdg-open`, `gtk-launch`, `systemctl`, the daemon and its `ffmpeg`)
      run without the bundle's library paths or the injected `GDK_BACKEND`
- [x] The app stops the daemon it started on exit with SIGTERM, a 10 s grace period, then SIGKILL;
      installed or pre-existing daemons are left running
- [x] Release CI checks that the AUR package no longer carries WebKit

## Acceptance Criteria

- [x] Host tools started from the AUR package or the AppImage get the host environment, so `gdbus` no longer
      fails on the bundled glib
- [x] No WebKit abort on exit from the AUR package
- [x] Quitting the app stops a daemon it spawned, and the daemon exits with code 0
- [x] Next tagged release publishes the new `wingdrive-bin` through the AUR job

## Evidence

- `cargo test -p file-opening-linux`: 3 host environment tests pass.
- `cargo test -p wingdrive --bin WingDrive`: 7 pass. The daemon integration test now asserts exit code 0
  after SIGTERM.
- Local `makepkg` of the new PKGBUILD over the alpha.8 AppImage: package goes from 477 MiB to 198 MiB,
  `ldd` reports no missing library, and `WebKitWebProcess` loads from `/usr/lib/webkit2gtk-4.1`.
- `scripts/release/smoke.py` on the package: daemon replies, window visible, 15 tabs open. Memory was
  557 MiB on this host; the alpha.8 AppImage measured 600 MiB on the same host.
- `gdbus call` under the AppImage environment reproduces the symbol error. Under the package
  environment the same call succeeds.
- Debug app run with a simulated bundle environment (`APPDIR`, `LD_LIBRARY_PATH`, `XDG_DATA_DIRS`):
  the spawned daemon's `/proc/<pid>/environ` has no `APPDIR` or `LD_LIBRARY_PATH`, and `XDG_DATA_DIRS=/usr/share`.
  Closing the window logged `App exiting, stopping daemon we started`. The daemon printed
  `Received SIGTERM, shutting down gracefully...` and exited within 100 ms, and the app exited with 0.
- 2026-10-08: v2.0.0-alpha.9 published to AUR (`a4c9ce9..9ba81bc`, run 37799386737). The Arch smoke test on the system WebKit measured cold start to daemon ready 2.8 s and 754.3 MiB PSS with 15 tabs (WingDrive 161.0, WebKitWebProcess 473.9, WebKitNetworkProcess 40.0, wing-daemon 79.4). The AUR job now has its own 850 MiB budget (`WINGDRIVE_AUR_MEMORY_BUDGET_MIB`); the bundled-WebKit AppImage stays at 550 MiB (511.0 MiB measured).
