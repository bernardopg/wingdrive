---
id: DESK-036
title: Open in a new tab from the sidebar and folders, and drop files on tabs
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, explorer, tabs, dnd]
last_updated: 2026-10-28
---

## Description

Found by the author while testing 2.2.0 on Wayland. Browsers and Dolphin open a link or folder in a
new tab with a middle-click or Ctrl+click, which makes copying and moving between places quick.
WingDrive's sidebar (volumes, devices, locations) and its folders had no way to do that. Moving a
file to another tab also felt broken: the tab only switched after a full second of hovering, gave
no feedback while waiting, and dropping on the tab itself did nothing. Ctrl+L also read a name
without a leading slash from `/`, so `Downloads` opened `/Downloads`.

## Implementation Steps

1. `createTab` takes `{background: true}`; `OpenFileProvider` exposes a stable
   `useOpenInNewTabAction`, reading the tab manager through a ref so items do not re-render on
   navigation.
2. Middle-click on a folder (grid, list, column) and middle-click or Ctrl+click on sidebar items
   and network mounts open them in a background tab titled with the item's label.
3. A tab under dragged files shows an accent ring and switches after 400 ms; dropping on a tab opens
   the copy/move dialog for the folder that tab shows (skipped when the files are already there).
4. Ctrl+L resolves names relative to the folder shown, with `.` and `..`.
5. `wingPathFromParam` reads Explorer URLs without decoding them twice, which made folders with `%`
   in their name fail to open and their tab titles fall back to "Explorer".

## Acceptance Criteria

- [x] Middle-click and Ctrl+click on volumes, devices, locations and network mounts open a new tab
- [x] Middle-click on a folder opens it in a new tab
- [x] Dragging files over a tab highlights it and switches within half a second
- [x] Dropping files on a tab moves or copies them into that tab's folder
- [x] Ctrl+L with `Downloads` or `../x` resolves from the current folder
- [x] Verified on the Linux desktop build

## Evidence

- Unit tests: `pathInput.test.ts` (relative, `..`, `~`), `wingPathParam.test.ts` (`%` names, double
  encoding, tab folder), `OpenFileProvider.test.tsx` (one stable action, background tab).
- Runtime, 2026-10-28, debug build on a private Xvfb with an isolated daemon: middle-click on the
  Root volume and Ctrl+click on a location opened two background tabs; middle-click on `alvo` and
  `cem 100%` opened both (the `%` folder lists normally); dragging `mover.txt` to the `alvo` tab
  showed the ring, and dropping opened the dialog with `alvo` as destination; Move moved the file;
  Ctrl+L `../Downloads` from `cem 100%` opened the sibling `Downloads`.
- Dev builds now prefer the plain `target/debug/wing-daemon` over the triple-suffixed copy
  tauri-build leaves in `target/debug`, which had run a daemon from 2026-10-18 in the author's
  Wayland test (the source of its "Failed to update job status" errors).
