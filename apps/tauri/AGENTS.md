# AGENTS.md - WingDrive desktop (Tauri)

Read the root `AGENTS.md` first. This file adds the rules for `apps/tauri/`.

- Run the app with `just dev-desktop` (`bun run tauri:dev`); it builds and starts its own daemon.
  `WINGDRIVE_INSTANCE=<name>` runs an isolated instance with its own daemon, data and, on Windows,
  its own single-instance identifier and WebView2 profile.
- `src-tauri` must not depend on `wing-core` (CI checks it). Shared code goes in `crates/` or in
  `apps/tauri/crates/` (`file-opening-*`, `file-clipboard-windows`).
- Platform features are declared in `src/platform.ts`. Expose a feature only on the platforms that
  implement it: `runAsAdmin` is Linux-only, the file clipboard is Linux and Windows. A hidden
  action beats one that fails with "only available on Linux".
- Bundles: `tauri.conf.json`, plus `tauri.linux.conf.json` and `tauri.windows.conf.json`. A file the
  app loads at runtime (such as the toast logo `icons/128x128.png`) must be listed as a resource,
  and `scripts/release/windows_smoke.ps1` should check it is installed.
- Notifications: freedesktop on Linux (portal under Flatpak), toasts under the installer's
  AppUserModelID on Windows (`src-tauri/src/notifications.rs`).
- Test with `cargo test -p wingdrive --bin WingDrive`. `admin::tests::pkexec_round_trip_on_root_owned_files`
  is ignored; `scripts/check-admin-retry.sh` runs it against the real polkit.

<!-- headroom:memory-instructions -->
## Memory

Use the `headroom_memory` MCP server for persistent cross-session knowledge.

**Before** answering questions about prior decisions, conventions, project context,
architecture, user preferences, org info, codenames, debugging history, or anything
from past sessions — call `memory_search` first.

**After** making durable decisions, discovering conventions, or learning important
facts — call `memory_save` to persist them for future sessions.

Memory is your first source of truth for anything not visible in the current conversation.
