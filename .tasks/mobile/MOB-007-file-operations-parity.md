---
id: MOB-007
title: "File operations parity with the desktop"
status: In Progress
assignee: bernardopg
parent: MOB-000
priority: Medium
milestone: M5
sprint: S13
tags: [mobile, interface, parity]
last_updated: 2026-11-17
---

## Description

The mobile Explorer only lists. The desktop offers copy, move, rename, delete, new folder, tags,
favorites, trash and space editing through `files.*`, `tags.*`, `metadata.set_favorite`, `trash.*`
and `spaces.*` actions.

## Acceptance Criteria

- [ ] Rename, delete (to trash), new folder, copy and move from the Explorer context menu
- [ ] Favorite and tag a file; Favorites and Trash screens
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
- Still to do: copy and move (need a destination picker), favorite and tag from the sheet (only
  indexed entries can be favorited), a Trash screen.
