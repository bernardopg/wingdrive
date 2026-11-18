---
id: MOB-007
title: "File operations parity with the desktop"
status: Done
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S13
tags: [mobile, interface, parity]
last_updated: 2026-11-18
---

## Description

The mobile Explorer only lists. The desktop offers copy, move, rename, delete, new folder, tags,
favorites, trash and space editing through `files.*`, `tags.*`, `metadata.set_favorite`, `trash.*`
and `spaces.*` actions.

## Acceptance Criteria

- [x] Rename, delete (to trash), new folder, copy and move from the Explorer context menu
- [x] Favorite and tag a file; Favorites and Trash screens
- [x] Listings refresh after each action

## Progress (2026-11-17)

Checked on the moto g(7) with the files in `/sdcard/WingDriveTest`:

- Tap opens a file of this phone in the app the user picks (`FileProvider` of the native module,
  `content://<app id>.wingdrive.files/...`); the gallery showed `gradiente.jpg`.
- Long press opens Open with, Share, Rename and Delete. Share showed the system sheet with
  `planilha.csv`. Rename keeps the extension and selects only the name (`plasma.png` to
  `fractal.png`). Delete asks first and deletes permanently: Android has no trash for plain files,
  and the core's `move_to_trash` is unsupported there.
- New Folder in the header menu created "Criada no app".
- The Favorites screen exists (Recents and Favorites wing items open library-wide lists).
- Copy and Move hold a local file or folder while you browse to the destination. On Android 15
  with the review debug APK, copying a file preserved its source and checksum. Moving another
  file removed it from the source on disk and in the Explorer. Both appeared in the destination
  without a manual refresh. A copy into a folder with an existing file of the same name created a
  numbered copy. The paste bar blocked moving a folder into itself.
## Completion (2026-11-18)

Validated on the Moto G7 running Android 15 with the review debug APK:

- The file menu opens a sheet to add or remove favorites and create, apply, or remove tags for
  files in indexed locations. A favorite appeared in the Favorites view and a newly created tag
  was applied immediately. An existing tag could be applied and removed.
- Move to Trash on Android keeps the item on its storage volume. The Lixeira shortcut in Explore
  opens the list. A file trashed before an app update was restored to its original path with
  its contents intact. Permanently deleting it then removed the payload and the list entry.
- An unindexed folder was restored with its nested file intact. A host regression test checks
  that restore refuses an occupied destination.
- Trash actions cover local Android storage with write access. The records belong to this app
  installation and survive app restarts and updates.
