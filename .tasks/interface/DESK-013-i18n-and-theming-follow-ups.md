---
id: DESK-013
title: Finish i18n and theming for the Linux desktop
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [i18n, themes, desktop, linux]
last_updated: 2026-11-05
---

## Description

PR #201 translated the interface to Brazilian Portuguese (i18next, Crowdin),
added 12 themes, light/dark icons and a redesigned Settings, and moved Linux to
the in-app context menu. Native and unverified parts remain.

## Implementation Steps

1. Translate the tray menu (`apps/tauri/src-tauri/src/background.rs`: "Open
   WingDrive", "New Tab", "Quit WingDrive") from `preferences.language`, and
   rebuild it when `ConfigChanged { field: "language" }` arrives.
2. Open external links on the About page with `platform.openLink` instead of
   `<a target="_blank">`, which stays inside the webview in Tauri.
3. Proofread the Portuguese strings in Crowdin and fix wording that reads as a
   literal translation.
4. Verify the packaged AppImage on Hyprland: themes, light icons, context menu
   position and no menu bar.
5. Verify native context menu position on macOS and Windows (cursor position is
   now passed to `menu.popup`).

## Acceptance Criteria

- [x] Tray menu follows the UI language
- [x] About links open the system browser from the desktop app
- [x] pt-BR proofread to 100% in Crowdin (2026-11-05, see Evidence)
- [x] AppImage verified on Linux; macOS/Windows menu position checked or
      recorded as blocked on hardware (Linux on Hyprland, 2026-11-05; Windows menu position is
      part of the TAURI-012 round; macOS has no build to check)

## Evidence

- 2026-11-05, Crowdin proofreading. Crowdin already showed pt-BR at 100% translated and 100%
  approved, but the approvals were the workflow's `auto_approve_imported` (1704 of them on
  2026-10-15), so nobody had read the strings. All 2160 strings (six interface catalogs and the
  mobile one) were read against English. Automated checks first: every key present in both
  languages, placeholders and plural forms identical, final punctuation, spacing. Then 46 fixes:
  - Grammar: two `_one` plurals that used the plural (`inspector.file.itemsSelected_one`,
    `navigation.redundancy.fileCount_one`); `explorer.archive.*` built "Falha ao Compactando" from
    a gerund label, now nouns ("Compactação falhou"); `navigation.trash.confirmPurge` did not agree
    with "Tudo na lixeira"; `common.fileOperation.completed` read "Copiado 3 itens", now
    "Copiado: 3 itens"; the explorer's undo label "renomear" showed as "Desfeito: renomear".
  - One word per concept: pairing is "parear" everywhere (the overview and devices said
    "vincular"), volume tracking is "rastrear" (the sidebar said "acompanhar"), backfill is
    "Sincronização inicial" (three different wordings before), "tira de miniaturas", "Profunda"
    for the deep indexing mode, "Esc" and "Espaço" for keys, "Conectado"/"Desconectado" for
    peers, "Pesquisa" instead of "Busca".
  - Wording: "Dividir a visualização" (was the European "vista"), "Próxima" page (was
    "Avançar"), "Regras de exclusão", "Arquivo compactado" as the default archive name,
    "Explorador de Arquivos" for Windows' file manager.
  The repository's `pt` files are the source of truth for the Crowdin workflow, which uploads
  them with auto-approval; it ran after the merge so Crowdin holds the reviewed strings.
- 2026-11-05, `v3.0.0` AppImage from the GitHub Release (bundled GTK and WebKit) on the author's
  Hyprland 0.57 session (Wayland, 1920x1080), isolated instance, driven through `tauri-driver` and
  WebKitWebDriver so the author's mouse and keyboard were never used; the windows were moved to an
  unused workspace.
  - No menu bar: the webview is 1034 px tall in a 1034 px window.
  - Context menu: a right click on `releases.mdx` at (1138, 484) opened the in-app menu at
    (1138, 487), at the pointer.
  - Themes: Light, Nord, Dracula, Solarized and Dark picked in the Settings window each reached the
    main window through the daemon (`<html>` class `vanilla-theme`, `nord-theme`, `dracula-theme`,
    `solarized-theme`, none) with the matching background.
  - Light icons: the Light theme switched the Explorer's icons to `Folder_Light` and
    `Document_Light`; every dark theme kept `Folder` and `Document`.
