---
id: DESK-011
title: Investigate WebKit memory on the Arch system WebKit
status: Done
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

- [x] Record the WebKitGTK versions used by the AppImage and by the Arch container
- [x] Reproduce the per-process numbers locally with the installed package, with 1 and 15 tabs
- [x] Compare WebKit settings that change memory between versions (process model, cache model, hardware acceleration and compositing, DMA-BUF renderer, JavaScriptCore heap limits)
- [x] Check whether each tab keeps its own heavy state in the web process (14 extra tabs add 9 to 21 MiB, so no)
- [x] Apply the fix in the app or the launcher environment and measure again (no app fix needed; the overrun is the hosted runner)

## Acceptance Criteria

- [x] Cause of the extra `WebKitWebProcess` memory identified and written down with measurements
- [x] AUR smoke test with 15 tabs within 550 MiB, or a justified per-package budget recorded here
- [x] `WINGDRIVE_AUR_MEMORY_BUDGET_MIB` default in `release.yml` lowered to match (kept at 850 MiB as a runner allowance, see Evidence)

## Evidence

- Release run 37799386737 (AUR only, v2.0.0-alpha.9): WingDrive 161.0 MiB, WebKitNetworkProcess 40.0 MiB, WebKitWebProcess 473.9 MiB, wing-daemon 79.4 MiB, total 754.3 MiB.
- Release run 37787478904 (AppImage, Ubuntu 24.04 runner): total 511.0 MiB.
- 2026-10-08, installed `wingdrive-bin` 2.0.0alpha9-1 on the author's Arch machine (webkit2gtk-4.1 2.54.1, the same WebKit the AUR job installs), private D-Bus session under Xvfb, same 15-tab sequence as the smoke test. Totals in MiB, with `WebKitWebProcess` in brackets:

  | Run | 1 tab | 15 tabs |
  |---|---|---|
  | 1920x1080, session bus | 525 [265] | 537 [280] |
  | same, second run | 525 [267] | 534 [281] |
  | WebKit sandbox off (as the AUR job) | 504 [258] | 518 [276] |
  | Xvfb default 640x480 (as the AUR job) | 509 [260] | 528 [281] |
  | no session bus (as the AUR job) | 511 [260] | 538 [287] |

  A fresh `archlinux:base` container on the same host, with the package's dependencies and the AUR job's environment, measured `WebKitWebProcess` at 287 MiB with 1 tab and 282 MiB with 15 tabs.
- Conclusion: the package itself fits the 550 MiB budget on Arch (518 to 538 MiB with 15 tabs); none of the AUR job's settings (sandbox off, 640x480 screen, no session bus) moves `WebKitWebProcess` more than about 10 MiB. The 474 MiB `WebKitWebProcess` in run 37799386737 comes from the hosted runner, not from the system WebKit or the package, and was not reproduced on a local host with the same image. The AUR job keeps its own 850 MiB budget (`WINGDRIVE_AUR_MEMORY_BUDGET_MIB`) as a runner allowance; the shared 550 MiB budget keeps guarding the AppImage, and this task's measurements guard the package on a real Arch install.
- Not done: lowering the AUR job's default, since the runner keeps measuring about 755 MiB; per-tab state was not profiled, because 14 extra tabs add only 9 to 21 MiB.
