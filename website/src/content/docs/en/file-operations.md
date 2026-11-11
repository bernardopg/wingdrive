---
title: File operations and undo
description: Copy, move, compress, checksums and the safety rails that protect your data.
order: 5
group: Using WingDrive
---

WingDrive has the full toolbox you expect from a file manager, and it is careful with your data.

## The toolbox

- **Copy To** and **Move To** any location, volume or paired device.
- **Compress** and **extract** archives from the context menu.
- **Templates**: create new files from your templates folder.
- **Dolphin service menus**: your existing KDE service menus show up in the context menu.
- **Open With**, including desktop entries with field codes such as `%f` and `%u`.
- **Permissions** and **SHA-256 checksums** from the Inspector.

## Undo

Copy, overwrite, cross-volume move and trash are journaled. Press **Ctrl+Z** in the desktop app, or run `wing file undo`, to reverse the last one. `wing file history` lists what can still be undone.

- A file that a copy replaces is kept for 7 days in a hidden `.wing-undo-*` folder next to it.
- Undo skips anything that changed since the operation, so it never overwrites newer work.
- The journal lives in each library's job database and is never synced.

## Safety rails

- **Copy preflight** checks free space and conflicts before anything is written.
- **Atomic copies**: files are written under a temporary name and renamed into place. A cancelled or failed copy leaves no truncated file behind and never destroys the file it was replacing.
- **No copies into themselves**: a folder can't be copied into its own subtree.
- **Timestamps** of files and folders are preserved.
- **Symbolic links** are copied, moved and renamed as links.
- **Secure delete** refuses copy-on-write filesystems (btrfs, ZFS, APFS, ReFS, bcachefs), where overwriting in place can't be guaranteed.
- **System mounts** are never ejected.

## Jobs

Long operations run as **jobs** that report progress, can be paused and cancelled, and resume after a restart. Watch them in the job manager or with `wing job monitor`.
