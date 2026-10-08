---
id: DESK-011
title: Investigate WebKit memory on the Arch system WebKit
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S03
tags: [desktop, performance, linux]
last_updated: 2026-10-08
---

## Description

The `wingdrive-bin` AUR package runs on Arch's system GTK and WebKit. Its release smoke test with 15 tabs measured 754.3 MiB PSS, against 511.0 MiB for the AppImage, which bundles Ubuntu's WebKit. Almost all of the difference is in `WebKitWebProcess`. The AUR job was given its own 850 MiB budget so alpha.9 could publish; this task finds the cause and brings the package back under the shared 550 MiB budget.

## Implementation Steps

- [ ] Record the WebKitGTK versions used by the AppImage and by the Arch container
- [ ] Reproduce the per-process numbers locally with the installed package, with 1 and 15 tabs
- [ ] Compare WebKit settings that change memory between versions (process model, cache model, hardware acceleration and compositing, DMA-BUF renderer, JavaScriptCore heap limits)
- [ ] Check whether each tab keeps its own heavy state in the web process (unmounted tabs, image caches, virtualised lists)
- [ ] Apply the fix in the app or the launcher environment and measure again

## Acceptance Criteria

- [ ] Cause of the extra `WebKitWebProcess` memory identified and written down with measurements
- [ ] AUR smoke test with 15 tabs within 550 MiB, or a justified per-package budget recorded here
- [ ] `WINGDRIVE_AUR_MEMORY_BUDGET_MIB` default in `release.yml` lowered to match

## Evidence

- Release run 37799386737 (AUR only, v2.0.0-alpha.9): WingDrive 161.0 MiB, WebKitNetworkProcess 40.0 MiB, WebKitWebProcess 473.9 MiB, wing-daemon 79.4 MiB, total 754.3 MiB.
- Release run 37787478904 (AppImage, Ubuntu 24.04 runner): total 511.0 MiB.
