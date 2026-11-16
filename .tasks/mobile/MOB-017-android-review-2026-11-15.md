---
id: MOB-017
title: "Android review on the moto g(7): what keeps the app from production"
status: To Do
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S07
tags: [mobile, android, ui, i18n, review]
related_tasks: [MOB-007, MOB-008, MOB-015, BRAND-003]
last_updated: 2026-11-15
---

## Description

Review asked by the author on 2026-11-15 on the test phone (moto g(7), LineageOS, Android 15),
updated that day from `v2.1.0` to the signed `v3.3.1` APK of the GitHub release. The library
survived the update, the two new migrations ran, and the core started without WARN or ERROR in
logcat. The Network screen works: P2P on, the desktop paired and connected, with its model, OS and
version. Settings reads well and is fully translated.

What is wrong, most important first:

1. **The phone cannot see its own files.** The app requests no storage permission (only
   `INTERNET`, `ACCESS_NETWORK_STATE`, `CAMERA` and `VIBRATE`). The volume manager finds 0 volumes,
   the Overview shows "0 B" of storage, and opening "motorola moto g(7)" in Explore shows "Nenhum
   item". A file manager that shows no files on its own device is not usable. MOB-008 (locations
   through the Storage Access Framework) is the planned answer; it also needs at least the shared
   storage volume (`/storage/emulated/0`) listed through `MediaStore` or SAF.
2. **A paired desktop shows no files.** "Bitter Arch Hypr DMS" appears as Online in Explore, but
   opening it shows "Nenhum item", and the Overview lists it with "0 volumes". The two devices are
   paired but do not share a library, so the phone has nothing to show. The screen should say so
   and offer to share the library, instead of an empty list.
3. **Untranslated text in pt-BR:** "Files Indexed", "0 B used", "All Devices" (the default wing's
   name, which should be localized when it is the seeded default) and "SOURCES".
4. **Invented number on the Overview:** "70 TOPS, Poder computacional de IA, Muito alto" on a
   Snapdragon 632 with no NPU: `HeroStats.tsx` hardcodes `const topsValue = 70`. The card should
   show a measured value or nothing.
5. **Layout:** after scrolling the Overview, the large "My Library" title and the header buttons
   draw under the status bar clock and icons (MOB-015 fixed this for the first screen only).
6. **Disabled search:** the search field on the Overview and Explore is visible but disabled, with
   no hint why.
7. **Slow taps:** switching tabs or opening a device takes 1 to 3 seconds before the screen changes
   on this phone.
8. **"Criar espaço" does nothing.** The last page of Explore shows a "+" and "Criar espaço", but
   `CreateWingScreen` has no handler: tapping it does nothing. Either open a create form (name,
   icon, color, then `wings.create`) or remove the page. A visible control that does nothing breaks
   the rule that every action is real, unavailable or hidden.
9. **Wings from the desktop sync, but half of them stay invisible.** A wing, a group and two items
   created on a desktop on `main` reached the phone in seconds ("Shared change applied successfully"
   for `space_group` and `space_item`). The phone shows the wing and the group, but hides Recents and
   Favorites items because they have no mobile screen (MOB-007), so the group looks empty.
10. **Wings (BRAND-003):** `v3.3.1` still says "Criar espaço" / "espaços"; `main` says Wing. Comes
    with the next release.

## Acceptance Criteria

- [ ] Items 1 and 2 have tasks of their own or are closed by MOB-008, with the phone showing its
      shared storage and a paired desktop's files.
- [ ] Items 3 to 9 fixed and checked on the moto g(7) with screenshots.
