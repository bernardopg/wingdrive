---
id: TAURI-011
title: Verify Linux Production Bundle
status: Done
assignee: bernardopg
parent: TAURI-000
priority: High
sprint: S01
milestone: M1
tags: [tauri, packaging, linux]
last_updated: 2026-10-02
---

## Description

Verify production Linux bundles with isolated daemon startup. The user explicitly deferred TAURI-006; its broader regression matrix remains outside this task.

## Acceptance Criteria

- [x] `bun run tauri build` produces AppImage and deb on Linux
- [x] Installed bundle starts the packaged daemon and connects the main window
- [x] Packaged startup smoke check passes against the production bundle
- [x] Wayland startup works or the GDK_BACKEND=x11 fallback is applied automatically

Verification (2026-10-02): alpha.5 AppImage (205.87 MiB) and deb (75.98 MiB) built successfully. scripts/release/smoke.py verified the stripped packaged daemon, isolated RPC startup and visible desktop window, then measured 472.2 MiB with 15 tabs. With DISPLAY available and GDK_BACKEND unset, startup prefers XWayland through x11,wayland. GitHub and AUR publication remains in FORK-004.

Installed Arch verification (2026-10-03): a clean pacman installation of wingdrive-bin starts through /usr/bin/wingdrive as a normal user, connects to the packaged daemon and opens the main window. The same smoke verifier measured 450.2 MiB PSS with 15 tabs. Extracted directory modes are normalized before packaging and the release workflow repeats this installed runtime check.
