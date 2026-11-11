---
title: First run
description: Create a library, add your first locations and learn the interface.
order: 3
group: Start
---

WingDrive runs as a background service (the **daemon**) with a separate interface. Closing the window does not stop indexing or sync, and with *Keep running in the background* enabled WingDrive stays in the tray so folders open instantly.

## 1. Create a library

On first launch the setup wizard asks for a library name, such as "Personal" or "Work". You can create more libraries later and switch between them.

The library is a local database. Backing up your whole organizational system is as simple as copying the library folder.

## 2. Add locations

A location is a folder WingDrive watches. Click **+** next to *Locations* in the sidebar, or go to **Settings → Locations**, and pick a folder. Start with something manageable, like Documents or Pictures.

Indexing modes:

- **Shallow**: names and basic metadata only. The fastest.
- **Content**: adds content identity (BLAKE3), so duplicates and copies are recognized.
- **Deep**: adds media analysis such as thumbnails. Recommended for photo and video folders.

Watch progress in the job manager, from the status indicator. Removable drives you plug in show up as volumes automatically.

## 3. Find your way around

- **Sidebar**: overview, recents, favorites, locations, volumes, devices and tags. Customize groups to fit how you work.
- **Explorer**: the file view, with tabs, grid, list, columns, media and size views.
- **Inspector**: metadata, tags and technical details of the selection (Ctrl+I).
- **Search bar**: finds files across every indexed location.
- **Command palette**: every action by name (Ctrl+Shift+P).

## 4. Make it your default

On Linux, WingDrive can become the default file manager, so `xdg-open` on a folder and "Show in folder" in other apps open it. See [Linux integration](/docs/linux-integration/).

## Next

- [Use the Explorer](/docs/explorer/)
- [File operations and undo](/docs/file-operations/)
