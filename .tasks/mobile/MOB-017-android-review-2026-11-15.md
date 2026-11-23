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
last_updated: 2026-11-23
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
6. ~~Disabled search~~: not a bug. The field is a button that opens the search screen; the
   review tapped it later and it worked.
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

## Progress (2026-11-16)

- Item 1: the core detects Android volumes (`core/src/volume/platform/android.rs`): shared storage
  `/storage/emulated/0` as "Internal storage" and each card under `/storage/<id>`, with capacity from
  `statvfs`. The app declares `MANAGE_EXTERNAL_STORAGE`, the native module answers
  `hasAllFilesAccess()` and opens the system page with `openAllFilesAccessSettings()`, and Explore
  shows an "Allow access to files" card until the user grants it (checked again when the app comes
  back to the foreground). The empty view of this device shows the same card.
- Item 2: a paired device that shares no folders shows "This device shares no folders with this
  library yet..." instead of "No items".
- Item 3: "Files Indexed", "used", "Sources", "All Devices" and "Internal storage" translated.
- Item 4: the AI compute card is gone; the first page shows storage, files, devices and library size.
- Item 5: the title row fades before it reaches the status bar.
- Item 8: "Create New Wing" opens a name prompt and creates the wing with `wings.create`.

## Checked on the moto g(7) (2026-11-16)

Release build of PR #399 installed beside the everyday app as `com.wingdrive.app.review`
(debug-signed), fresh library:

- Internal storage detected at start ("New volume discovered: Internal storage", 50 GB, 46 GB free)
  and shown on the Overview under the phone.
- Explore shows "Allow access to files"; "Open settings" opens the system page "Acesso a todos os
  arquivos" for WingDrive. After granting it and coming back, the card is gone without a restart.
- Opening "Internal storage" lists the phone's folders (Alarms, DCIM, Download, Pictures...) and
  subfolders open.
- "Create New Wing" asks for a name and creates the wing ("Trabalho", orange).
- Portuguese: "Arquivos indexados", "usado", "Armazenamento interno" translated; the title row fades
  away when the Overview scrolls, so nothing draws under the status bar.

Still wrong, found in this check:

1. The folder view of the internal storage is titled "0" (the last segment of
   `/storage/emulated/0`); it should show the volume name.
2. "Total Storage" stays at 0 B while the volume is not tracked; the phone's own storage should count.
3. Volume chips show the filesystem as "android" and "Primary", and the free space says "free":
   untranslated or meaningless labels.
4. A fresh install starts in English on a phone set to pt-BR; the app should follow the system
   language until the user picks one.
5. The core logs `WARN swarm_discovery::socket: error sending mDNS: Operation not permitted` every
   few seconds: Android needs a Wi-Fi multicast lock (`CHANGE_WIFI_MULTICAST_STATE`) for mDNS, so
   phones do not find desktops on the local network by discovery.
6. A new wing is an empty page with no hint on how to add items.
7. The system page shows the app as version 1.0.0 (MOB-009).
8. Item 7 of the review (taps taking 1 to 3 s) still has no cause: `gfxinfo` shows no slow frame,
   so the next step is a Hermes profile of the JS thread.

## Progress (2026-11-17)

Pull requests for the items of "Still wrong", not yet checked on the moto g(7):

- Items 1 to 3: #419. The folder view takes the item's name as its title ("Internal storage"),
  the volume row hides the "android" placeholder and shows Internal, External, Network or Cloud,
  "free" is translated, and Total Storage counts the detected volumes when the library tracks none.
- Item 4: #416. The app follows the phone's language until the user picks one in Settings, which
  gains "System default".
- Item 5: #417. The native module holds a Wi-Fi multicast lock while the core runs and declares
  `CHANGE_WIFI_MULTICAST_STATE`.
- Item 7: #418. Every Android build carries the product version from `app.json` (MOB-009).

## Checked on the moto g(7) (2026-11-17)

Debug build of `main` plus this branch, installed as `com.wingdrive.app.review`:

- #416, #417, #418 and #419 hold: the app starts in Portuguese, Internal storage (50 GB) is on the
  Overview with "Armazenamento interno", "Interno" and "livres", and the folder of the volume is
  titled with its name when opened from the Explore volumes list (that path still said "0").
- Item 6: an empty wing explains that its items are added from the desktop sidebar.
- Item 9: Recents and Favorites items open library-wide lists.
- "My Library" and "All Devices" show translated while the user keeps the seeded names.
- File names are no longer translated (a folder named Notifications showed "Notificações"), and
  names show their extension.
- Images and videos on the phone show previews.
- Still open: item 7 (slow taps). In the debug build a long press needs more than a second and
  taps lag; check again in a release build before profiling.

## Checked on the moto g(7) (2026-11-22)

Device: `ZY225DFMGB` (moto g(7), Android 15, 1080x2270). Installed app: `com.wingdrive.app`
3.6.0 (versionCode 3060099), debug dev client connected to Metro running from the main
`wingdrive` repo (`apps/mobile`, `bunx expo start`, port 8081). No rebuild/install performed;
validation used the existing dev client and JS served by Metro on `origin/main` (this branch had
no code changes at validation time).

Evidence (screenshots/OCR in `/tmp/mob017/evidence/` on the host, not committed): `overview-top.*`,
`overview-mid.*`, `browse.*`, `archlinux.*`.

- **Item 1 (phone sees own files):** Partially met. Overview shows Internal storage (50.0 GB,
  SSD, Internal, 46.8 GB free). Browse shows "Permitir acesso aos arquivos" card and storage/volume
  entries. The all-files access flow and Android volume detection are in code (progress 2026-11-16).
  MOB-008 remains the planned SAF/locations answer; this item is not fully closed by MOB-008 alone
  because full location management is still tracked there.
- **Item 2 (paired desktop shows files):** Partially met. A paired desktop that shares no folders
  shows the explanatory message (code fix present). On this check `archlinux` was Offline but its
  volumes (`home`, `Root`) were listed in Explore; file browsing while online was not validated in
  this session because the paired desktop was disconnected. AC1 cannot be marked fully done.
- **Item 3 (untranslated pt-BR):** Looks fixed in current build. Overview showed "Armazenamento
  total", "Arquivos indexados", "610.0 GB usado", "arquivos únicos", "Dispositivos conectados",
  "Tamanho da biblioteca", "registrados na biblioteca", "banco de dados no disco", "MINHA REDE",
  "AÇÕES RÁPIDAS". Browse showed "Todos os dispositivos", "Permitir acesso aos arquivos",
  "Recentes", "Favoritos", "Lixeira", "DISPOSITIVOS". Volume labels showed "Interno" and "livres".
- **Item 4 (invented TOPS):** Fixed in code; HeroStats has no TOPS/AI compute card.
- **Item 5 (layout under status bar):** Code applies hero title-row fade (`heroTitleRowStyle`).
  Visual confirmation was limited by OCR-only inspection; prior 2026-11-16/17 checks marked it ok.
- **Item 6 (disabled search):** Not a bug (confirmed earlier).
- **Item 7 (slow taps):** Still open. Debug+Metro measurements on device (screen-change after tap,
  3-4 samples): tab switches Overview/Browse/Network/Settings ~0.78-0.84 s; opening `archlinux`
  device from Browse ~0.81 s. These are below the original 1-3 s report but still noticeable, and
  they were taken on a debug dev client served by Metro, not a release build. Per task note, check
  again in a release build before profiling. No code fix attempted.
- **Item 8 (Criar espaço does nothing):** Fixed in code (`CreateWingScreen` prompts for name and
  calls `wings.create`). Not re-exercised on device in this session due environment quota blocking
  new screenshot/OCR artifacts; prior 2026-11-16 check confirmed it.
- **Item 9 (wings sync but items invisible):** Prior 2026-11-17 check marked Recents/Favorites
  opening library-wide lists; not re-validated in this session.

## Acceptance Criteria Status (2026-11-22)

- [ ] Items 1 and 2 have tasks of their own or are closed by MOB-008, with the phone showing its
      shared storage and a paired desktop's files. *Shared storage shows; paired desktop file
      browsing not validated while online this session; MOB-008 still tracks full SAF/locations.*
- [ ] Items 3 to 9 fixed and checked on the moto g(7) with screenshots. *Item 7 remains open;
      items 3,4,5,6,8,9 have prior evidence/code fixes but this session did not re-screenshot every
      item due env quota constraints; item 7 measured in debug+Metro only.*

## Realignment (2026-11-23)

- PR #493 merged with device validation notes on moto g(7). AC1 (paired desktop files online) and AC2 (item 7 release-build/profile) remain unclosed; no code fix for item 7 without release profiling. Kept To Do.

## Progress (2026-11-23, evidence collection)

Device: `ZY225DFMGB` (moto g(7), Android 15, 1080x2270). Installed app: `com.wingdrive.app` 3.6.0 (versionCode 3060099), **debuggable** build (`dumpsys package` flags include DEBUGGABLE). Host evidence: `/tmp/opencode/mob017/*.png` + OCR via `tesseract` (not committed).

Observed:

- **Item 1 (storage/own files):** Overview shows "Armazenamento interno 50.0 GB, SSD Interno 46.8 GB livres" under motorola moto g(7). Browse shows "Permitir acesso aos arquivos" card and device/volume entries. MOB-008 is Done; shared storage visible.
- **Item 2 (paired desktop files):** Network shows archlinux paired, Disconnected, v3.6.0. Browse shows archlinux Desconectado; opening it shows Root/home volumes. Full online file browsing not validated in this session (desktop offline). AC1 remains open.
- **Item 3 (pt-BR):** Overview/Browse/Network/Settings labels observed in pt-BR (Armazenamento total, Arquivos indexados, Dispositivos conectados, Tamanho da biblioteca, MINHA REDE, Interno, livres, Todos os dispositivos, Permitir acesso aos arquivos, Recentes, Favoritos, Lixeira, Rede, P2P ativado, Dispositivos pareados, Configurações).
- **Item 4 (invented TOPS):** No TOPS/AI compute card visible on Overview; HeroStats fix remains in code.
- **Item 5 (status bar layout):** Hero title-row fade present in code; no visual regression observed in OCR/layout checks, but no new pixel-level screenshot assertion added.
- **Item 7 (slow taps):** Not re-measured in release build. Installed app is debuggable dev client; prior 2026-11-22 debug+Metro measurement was ~0.78-0.84s. Release-build profile still required before closing.
- **Item 8 (Criar espaço):** Prior fix present; not re-exercised this session.
- **Item 9 (Recents/Favorites):** Recents shows "Arquivos indexados recentemente nesta biblioteca aparecem aqui."; Favorites shows "Arquivos marcados como favoritos em qualquer dispositivo desta biblioteca aparecem aqui."

## Blockers

- AC1 cannot be fully closed without a paired desktop online and sharing files/library content visible on the phone.
- AC2 item 7 cannot be closed without release-build measurement/profiling on moto g(7).

## Progress (2026-11-23, final pass)

- Homelab (arch-legacy, wingdrive server docker host-network :8095), laptop (real wing-daemon running) and moto g(7) ZY225DFMGB (192.168.15.12, same LAN) available. Phone Network screen shows archlinux paired but Disconnected; laptop daemon shows moto g(7) offline. No online paired-desktop file browse validated this session.
- Release-build profiling (item 7): v3.5.0 release APK download available but cannot be installed over 3.6.0 debug without uninstall/signature mismatch (would wipe data). Local `cargo xtask build-mobile` release failed due Android NDK/linker env (`-llog` with `cc`). AC2 item 7 remains blocked for release-build evidence.
- COV/CI: #512 merged after Explorer smoke rerun (failure was unrelated server-log WARN). No open PRs at finalize time.

## Remaining blockers

- **AC1**: needs paired desktop online and sharing files/library visible on moto g(7). Pairing exists; runtime connectivity/file-sharing evidence still missing.
- **AC2 item 7**: needs release-build tap/profile on moto g(7) with safe install path (signed release or separate package/debug-signable release).
