---
title: Linux integration
description: Make WingDrive your default file manager, start it with your session and keep it in the tray.
order: 8
group: Using WingDrive
---

WingDrive's current milestone (M2) is to be your **default file manager** on Linux. It integrates with the desktop the way Dolphin or Nautilus do.

## Default file manager

Set WingDrive as the handler for folders:

```bash
xdg-mime default wingdrive.desktop inode/directory
xdg-mime query default inode/directory   # should print wingdrive.desktop
```

Then:

- `xdg-open ~/Downloads` opens the folder in WingDrive.
- **Show in folder** in browsers and other apps opens WingDrive with the file selected. WingDrive answers the `org.freedesktop.FileManager1` D-Bus interface while it is the default handler for `inode/directory`.

To go back, point `inode/directory` at your previous file manager, for example `org.kde.dolphin.desktop` or `org.gnome.Nautilus.desktop`.

## File chooser for uploads and saves

WingDrive can be the dialog that opens when you upload a file in a browser or choose where to save a download. Chrome, Firefox, Flatpak apps and GTK4 apps use it through `xdg-desktop-portal`.

In **Settings**, under **File chooser**, turn on **Use WingDrive to pick files in other apps**. WingDrive puts itself first in your portal configuration (`~/.config/xdg-desktop-portal/hyprland-portals.conf` on Hyprland, `portals.conf` elsewhere), keeps your previous chooser after it, and restarts the portal. Restart open browsers so they see the change. Turning the switch off puts the previous chooser back.

To edit the file yourself, add this line under `[preferred]`, with your previous chooser after `wingdrive`:

```ini
org.freedesktop.impl.portal.FileChooser=wingdrive;gtk
```

The packages install `wingdrive.portal`, which the switch needs. A build from source must install it by hand.

## Background and tray

In **Settings**, under **Background**:

- **Keep running in the background**: closing the window keeps WingDrive in the tray, so folders open instantly.
- **Start at login**: start WingDrive hidden in the tray when you log in.

## Terminal

F4 docks a terminal under the Explorer, and Shift+F4 opens your own terminal in the current folder. Set the command in **Settings**. Left empty, WingDrive uses `xdg-terminal-exec`, then `$TERMINAL`, then an installed terminal.

## Service menus and Open With

Your KDE/Dolphin **service menus** appear in the context menu. **Open With** lists applications from their desktop entries, and passes files correctly to entries that use field codes like `%f`, `%F`, `%u` and `%U`.

## Wayland

If the window fails to start on a Wayland session with a GTK error, run it through XWayland:

```bash
GDK_BACKEND=x11 wingdrive
```
