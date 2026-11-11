---
id: DESK-051
title: WingDrive as the file chooser for uploads and saves (xdg-desktop-portal FileChooser)
status: To Do
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S05
tags: [desktop, linux, portal, dbus, file-chooser]
last_updated: 2026-11-11
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

- [ ] **D-Bus backend:** serve `org.freedesktop.impl.portal.FileChooser` (version 4) on a WingDrive
  name such as `org.freedesktop.impl.portal.desktop.wingdrive`, with `OpenFile`, `SaveFile` and
  `SaveFiles`. Each request exports a `org.freedesktop.impl.portal.Request` object so the portal can
  close it.
- [ ] **Packaging:** ship `wingdrive.portal` (`DBusName`, `Interfaces`, `UseIn`) and a D-Bus
  activation file for the backend name, so a request works while WingDrive is not running. Check
  that the activation file does not collide with the `FileManager1` rule in `filemanager1.rs`:
  packages ship no activation file for `FileManager1`.
- [ ] **Picker window:** reuse the Explorer for navigation, search, sidebar and previews, with a
  bottom bar for Open, Save and Cancel. Support:
  - `multiple`, `directory`, `filters` and `current_filter`;
  - `current_folder`, `current_name` and `current_file` for saves;
  - `choices`;
  - `modal` and the parent window handle when the compositor allows it.

  Only local paths are returned, as `file://` URIs. A cloud or remote location in the picker is
  disabled or explained.
- [ ] **Save conflicts:** a Save over an existing file asks to overwrite.
- [ ] **Responses:** Cancel and closing the window return response 1, and errors return response 2,
  so the calling app never hangs.
- [ ] **Opt-in:** a Settings switch writes `org.freedesktop.impl.portal.FileChooser=wingdrive` to the
  user's `portals.conf` for the current desktop, and restores the previous value when turned off.
  Document the manual line for users who manage the file themselves.
- [ ] **Fallback:** when WingDrive or its daemon cannot start, the portal falls back to the previous
  backend instead of leaving the app without a dialog. Record how `xdg-desktop-portal` behaves when
  the configured backend fails to activate.
- [ ] **Docs:** `docs/` page on enabling it, and the AUR package notes.

## Acceptance Criteria

- [ ] With the switch on, Chrome's "upload file" opens the WingDrive picker. Choosing a file
  uploads it; Cancel leaves the page unchanged.
- [ ] Chrome's "Save as" and download location prompts open the picker in save mode, with the
  suggested name and folder. The file lands where chosen.
- [ ] Multiple selection, folder selection and type filters behave as the requesting app asks.
- [ ] A Flatpak app and a GTK4 app (portal path) use the picker too.
- [ ] Turning the switch off restores the previous FileChooser backend.
- [ ] Unit tests cover option parsing and responses; an integration test drives the D-Bus interface
  on a private session bus (`dbus-run-session`), as the FileManager1 tests do.
- [ ] Verified on the author's Hyprland session with the AUR package.

## Evidence

_None yet._
