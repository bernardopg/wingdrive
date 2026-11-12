---
id: DESK-051
title: WingDrive as the file chooser for uploads and saves (xdg-desktop-portal FileChooser)
status: In Progress
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S05
tags: [desktop, linux, portal, dbus, file-chooser]
last_updated: 2026-11-12
---

## Description

Picking a file to upload in Chrome, or choosing where to save a download, does not open WingDrive.
On Linux, Chrome, Firefox and Flatpak or GTK4 apps ask `xdg-desktop-portal` for a file chooser, and
the portal hands the request to the backend that `portals.conf` names for
`org.freedesktop.impl.portal.FileChooser`. WingDrive does not implement that backend, so another
dialog always answers. WingDrive replaces the file manager for browsing and for "Show in folder",
but these dialogs are where users meet a file manager most often, and today they never see it there.

WingDrive provides a FileChooser portal backend. A request opens a WingDrive picker window, which is
the Explorer with a confirm bar. The picker returns the chosen URIs to the portal, which passes them
to the app.

## Found on 2026-11-11, author's machine

- Hyprland session (`XDG_CURRENT_DESKTOP=Hyprland`). `xdg-desktop-portal` 1.22.1 with the `gtk` and
  `hyprland` backends. `~/.config/xdg-desktop-portal/portals.conf` and `hyprland-portals.conf` set
  `org.freedesktop.impl.portal.FileChooser=gtk`. No KDE portal and no `kdialog` are installed.
- Chrome runs on Wayland with `--gtk-version=4`. Its upload dialog comes from the portal FileChooser,
  not from WingDrive.
- Separate cause, fixed on the machine: "Show in folder" and opening a download's folder went to
  Dolphin. `inode/directory` still defaulted to `org.kde.dolphin.desktop`, so WingDrive left
  `org.freedesktop.FileManager1` alone (`filemanager1::should_serve`), and D-Bus activated
  `dolphin --daemon` through `org.kde.dolphin.FileManager1.service`. After
  `xdg-mime default wingdrive.desktop inode/directory` and a WingDrive restart, WingDrive owned the
  name, and a `ShowItems` call like Chrome's opened a WingDrive window.

## Implementation Steps

- [x] **D-Bus backend:** serve `org.freedesktop.impl.portal.FileChooser` (version 4) on a WingDrive
  name such as `org.freedesktop.impl.portal.desktop.wingdrive`, with `OpenFile`, `SaveFile` and
  `SaveFiles`. Each request exports a `org.freedesktop.impl.portal.Request` object so the portal can
  close it.
- [x] **Packaging:** ship `wingdrive.portal` with `DBusName`, `Interfaces` and `UseIn`. `UseIn`
  must list every desktop where WingDrive supports the picker (at least `Hyprland`, plus `KDE`,
  `GNOME` and others once checked); otherwise the portal never selects the backend there. Also ship
  a D-Bus activation file for the backend name, so a request works while WingDrive is not running. Check
  that the activation file does not collide with the `FileManager1` rule in `filemanager1.rs`:
  packages ship no activation file for `FileManager1`.
- [x] **Picker window:** reuse the Explorer for navigation, search, sidebar and previews, with a
  bottom bar for Open, Save and Cancel. Support:
  - `multiple`, `directory`, `filters` and `current_filter`;
  - `current_folder`, `current_name` and `current_file` for saves;
  - `choices`;
  - `modal` and the parent window handle when the compositor allows it.

  Only local paths are returned, as `file://` URIs. A cloud or remote location in the picker is
  disabled or explained.
- [x] **Save conflicts:** a Save over an existing file asks to overwrite.
- [x] **Responses:** Cancel and closing the window return response 1, and errors return response 2,
  so the calling app never hangs.
- [x] **Opt-in, which file:** a Settings switch edits
  `~/.config/xdg-desktop-portal/<desktop>-portals.conf`, where `<desktop>` is each entry of
  `$XDG_CURRENT_DESKTOP`, lowercased (`hyprland-portals.conf` on the author's machine). It edits
  `portals.conf` only when no per-desktop file exists, because the per-desktop file takes
  precedence: on the author's machine, a change to `portals.conf` alone would have no effect.
- [x] **Opt-in, which value:** the switch stores the previous `FileChooser` value, then writes an
  ordered list with WingDrive first and the previous backend after it, for example
  `org.freedesktop.impl.portal.FileChooser=wingdrive;gtk`. Turning the switch off writes the stored
  previous value back. Fallback and switch-off use the same stored value. Document the manual line
  for users who manage the file themselves.
- [x] **Fallback:** confirm from the `xdg-desktop-portal` 1.22 source and docs that a list is tried
  in order when the first backend cannot be activated, and record the result here. If it is not, the
  backend itself must hand the request on (or the switch must refuse to enable without a working
  activation), so an app is never left without a dialog.
- [x] **Docs:** `docs/` page on enabling it, and the AUR package notes. The same page covers "Show in
  folder": WingDrive only takes `org.freedesktop.FileManager1` when it is the default for
  `inode/directory` (`xdg-mime default wingdrive.desktop inode/directory`, or the Settings path that
  sets it). The code keeps not setting that default by itself, as `filemanager1.rs` explains.

## Acceptance Criteria

- [ ] With the switch on, Chrome's "upload file" opens the WingDrive picker. Choosing a file
  uploads it; Cancel leaves the page unchanged.
- [ ] Chrome's "Save as" and download location prompts open the picker in save mode, with the
  suggested name and folder. The file lands where chosen.
- [ ] Multiple selection, folder selection and type filters behave as the requesting app asks.
- [ ] A Flatpak app and a GTK4 app (portal path) use the picker too.
- [x] Turning the switch off restores the previous FileChooser backend. (Checked on temporary
  config folders in `file_chooser::tests::portals_conf`, not on the author's live file.)
- [ ] On the author's Hyprland session, the switch edits `hyprland-portals.conf` and the portal then
  selects the WingDrive backend.
- [ ] With the switch on, a stopped WingDrive whose activation fails still leaves Chrome's upload
  with a working dialog from the fallback backend.
- [ ] With `inode/directory` set to WingDrive, "Show in folder" from Chrome (`ShowItems`) opens a
  WingDrive window with the file selected.
- [x] Unit tests cover option parsing and responses; an integration test drives the D-Bus interface
  on a private session bus (`dbus-run-session`), as the FileManager1 tests do.
- [ ] Verified on the author's Hyprland session with the AUR package.

## Evidence

### Implementation (2026-11-12)

- Backend: `apps/tauri/src-tauri/src/file_chooser.rs` (broker, Tauri commands, switch commands),
  `file_chooser/portal.rs` (`org.freedesktop.impl.portal.FileChooser` on
  `org.freedesktop.impl.portal.desktop.wingdrive`, `/org/freedesktop/portal/desktop`, with
  `OpenFile`, `SaveFile`, `SaveFiles`, and one `org.freedesktop.impl.portal.Request` object per call
  at the portal's handle), `file_chooser/options.rs` (options, results, `file://` URIs, MIME globs),
  `file_chooser/portals_conf.rs` (the switch). `WINGDRIVE_FILE_CHOOSER=0` turns the backend off.
- Picker: a `file-chooser-<id>` window opens the Explorer at the start folder;
  `packages/interface/src/components/FilePicker/FilePickerBar.tsx` adds Open/Save/Cancel, the name
  field, filter and choices; `filePicker.ts` holds the pure logic. Closing the window answers 1.
- Switch: Settings > General > File chooser. It edits the first
  `~/.config/xdg-desktop-portal/<desktop>-portals.conf` with a `[preferred]` key, otherwise
  `portals.conf`, stores the previous value in `<data dir>/file-chooser-portal.json`, writes
  `wingdrive;<previous>`, and runs `systemctl --user try-restart xdg-desktop-portal.service`. Off
  writes the stored value back (or removes the line). It refuses to turn on without `wingdrive.portal`.
- Packaging: `apps/tauri/src-tauri/portal/wingdrive.portal` and the activation file
  `org.freedesktop.impl.portal.desktop.wingdrive.service` (`Exec=... --hidden`), installed by the
  `.deb` (`tauri.conf.json`) and the AUR PKGBUILD (`scripts/release/aur.py`). The activation file is
  for the backend name only; nothing ships for `FileManager1`. `wingdrive.portal` has no `UseIn`:
  the key is deprecated and would select WingDrive on desktops without `portals.conf` without the
  user opting in.
- Docs: `docs/core/linux-desktop-integration.mdx` (both services, manual line, fallback, package
  files) and the website's `linux-integration.md` (en and pt).

### Fallback, from the xdg-desktop-portal 1.22.1 source

`xdp_portal_config_find` (`src/xdp-portal-config.c`) walks the configured list in order and returns
the first entry whose `.portal` file is installed and lists the interface
(`xdp_portal_config_find_impl_config_by_iface`: "Requested backend %s does not exist. Skipping...").
It never checks whether the backend runs or can be activated, and `init_file_chooser`
(`src/file-chooser.c`) picks the backend once at portal start-up. So `wingdrive;gtk` falls back to
GTK only when `wingdrive.portal` is missing. Checked with the real 1.22.1 portal on a private bus
(`dbus-run-session`, `XDG_DESKTOP_PORTAL_DIR` pointing at a temp dir, `XDG_CURRENT_DESKTOP=Hyprland`,
`hyprland-portals.conf` with `FileChooser=wingdrive;gtk`):

- with `wingdrive.portal`: `Using wingdrive.portal for org.freedesktop.impl.portal.FileChooser (interface specific config)`;
- without it: `Requested backend wingdrive does not exist. Skipping...` then `Using gtk.portal for ...`;
- with `wingdrive.portal` and no backend on the bus: the portal still exports
  `org.freedesktop.portal.FileChooser` and accepts `OpenFile`, and no `Response` arrived within 4 s.

Mitigations: the activation file starts WingDrive for a request, and the switch refuses to turn on
without `wingdrive.portal`. A WingDrive that is installed but cannot start still leaves apps without
a dialog until the switch is turned off; the backend cannot hand the request on because it never
runs in that case.

### Tests

- `cargo test -p wingdrive --bin WingDrive file_chooser`: 38 pass. Covers option parsing (defaults,
  types, NUL-terminated paths, `current_filter` outside the list, `SaveFiles` names), results
  (single/multiple, folders, filters, overwrite confirmation, `SaveFiles` unique names, choices),
  URI encoding, MIME globs, response mapping (0/1/2), the broker (cancel, window closed, failed
  window), `portals.conf` editing and restore (per-desktop precedence, duplicate keys, symlinked
  files, missing backend), and `file_chooser::tests::bus`: five tests against a private
  `dbus-daemon` the test starts (name ownership, introspection, `OpenFile` URIs, `Request.Close`
  answering 1, parallel `SaveFiles`/`OpenFile`, invalid options and a failed window answering 2).
- `bun test packages/interface/src/components/FilePicker/filePicker.test.ts`: 11 pass.
- `cargo clippy -p wingdrive --all-targets -- -D warnings`, `cargo fmt --all`, `bun run typecheck`.

### Not verified here

- Chrome, Firefox, a Flatpak app and a GTK4 app driving the picker end to end, the Settings switch
  on the author's real `hyprland-portals.conf`, and the AUR package: these change the user's live
  portal setup and browser, which this run must not touch. The real-portal selection above was
  checked on a private bus only.
- The picker window was not rendered in a running Tauri build; typecheck and unit tests cover it.
- `modal` and `parent_window` are parsed but not applied; the picker opens as a normal window.
- Filters do not hide entries in the Explorer listing; a selection outside the chosen filter is
  refused with a message.
