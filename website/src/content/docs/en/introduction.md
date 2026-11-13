---
title: What is WingDrive?
description: A local-first file manager that indexes every drive you own into one library.
order: 1
group: Start
---

WingDrive is a desktop file manager with a background daemon that indexes your folders, external drives and cloud volumes into one **library**. Files stay where they are. The index is what makes them fast to browse, search and recognize across the devices you own.

There is no account, no telemetry and no central server. Indexing, thumbnails and search run on your machine.

> **Alpha.** Linux is the supported platform, and the current goal (milestone M2) is to replace Dolphin, Nautilus or Yazi as your default file manager. Windows installers ship as a beta since 3.0. macOS builds from source.

<figure class="shot">
<a href="/img/shots/WingGridView-1600.webp"><img src="/img/shots/WingGridView-1600.webp" srcset="/img/shots/WingGridView-800.webp 800w, /img/shots/WingGridView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="WingDrive grid view with the Inspector"></a>
<figcaption>One window for every folder, drive and cloud you own.</figcaption>
</figure>

## What you get

- **A fast Explorer** with grid, list, columns, media and size views, Quick Preview, a dual-pane mode (F3) and a docked terminal (F4).
- **Search** that refines by kind, extension, size, date and hidden files, with counts over the whole result.
- **Safe file operations**: copy preflight, atomic copies, undo for copy, move and trash, compress and extract, checksums.
- **Content identity**: every file gets a BLAKE3 hash, so the same file on two devices is one file.
- **Devices**: pair your own machines over Iroh/QUIC to sync the library and send files with Wingdrop.
- **Google Drive and Google Photos** with several accounts: browse, copy, edit in your own apps, sync folders and upload photos.
- **Cloud volumes** through OpenDAL: S3 and compatible services, Dropbox, OneDrive, Azure Blob and GCS.
- **`wing` CLI and `wing-server`** for scripts and headless machines, on the same daemon.

## Core ideas

**Library.** Your personal database of file information: metadata, tags, devices and sync settings. It lives on your machine and can sync its organizational data with devices you pair.

**Location.** A folder WingDrive watches and indexes. Adding a location never moves or copies files.

**Volume.** A storage device: an internal disk, a removable drive, or a cloud service mounted as a volume.

**Virtual filesystem (VDFS).** One view of files across every device and cloud. Each file keeps its physical location but appears in one searchable interface.

**WingPath.** A universal file address. A path can be *physical* (a file on a device), *cloud* (`s3://bucket/photo.jpg`) or *content* (find this file by what it is, wherever a copy lives).

## Next steps

- [Install WingDrive](/docs/installation/)
- [First run](/docs/first-run/)
- [Use the Explorer](/docs/explorer/)
