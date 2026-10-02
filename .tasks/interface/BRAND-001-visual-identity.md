---
id: BRAND-001
title: Replace the Upstream Visual Identity with WingDrive Branding
status: Done
assignee: bernardopg
parent: TAURI-000
priority: Medium
milestone: M1
sprint: S01
tags: [branding, design, assets, fork]
last_updated: 2026-10-02
---

## Description

FORK-003 renamed every identifier, but part of the visual identity is still the upstream one: the purple/blue glossy orb (app logo, favicon, Android launcher, iOS icon variants, startup and About screens, intro videos) and the upstream file-kind icon pack. The current WingDrive mark is the purple wing at `packages/assets/images/WingDriveLogo.svg` (same art in `.github/logo.svg`, `docs/logo/wingdrive-logo.svg`, and the Tauri icons).

Inventory made on 2026-10-01 by comparing each asset's last content change with the fork start (August 2026) and checking the images by eye.

## Needs New Artwork (owner: design)

### P1: Brand marks seen by every user

| File(s) | Today | Used by | Deliver |
|---|---|---|---|
| `docs/public/favicon.png` | Orb, dark tile | docs site (`docs/docs.json`, `docs/mint.json`) | Wing favicon PNG, 256x256 |
| `apps/mobile/android/app/src/main/res/mipmap-{mdpi,hdpi,xhdpi,xxhdpi,xxxhdpi}/ic_launcher.webp`, `ic_launcher_round.webp`, `ic_launcher_foreground.webp` (15 files) | Orb | Android launcher | Wing launcher set; regenerate from a 1024 px source with Expo/Android tooling |
| `apps/mobile/android/app/src/main/res/drawable-{mdpi..xxxhdpi}/splashscreen_logo.png` (5 files) | Expo placeholder grid | Android splash | Wing splash mark on transparent background |
| `apps/tauri/assets/exports/Icon-iOS-{Default,Dark,ClearDark,ClearLight,TintedDark,TintedLight}-1024x1024@1x.png` (6 files) | Blue orb | iOS/macOS icon variants (Icon Composer exports) | Six wing variants, 1024x1024 |
| `packages/assets/images/AppLogo.png`, `AppLogoV2.png` | Orb app icons | not referenced in code | Delete, or replace with wing app icon if a raster logo is needed |

### P2: In-app brand visuals

| File(s) | Today | Used by | Deliver |
|---|---|---|---|
| `packages/assets/images/Ball.png`, `BallBlue.png` | Purple and blue orb | `Settings/pages/AboutSettings.tsx`, `components/overlays/DaemonStartupOverlay.tsx`, `windows/VoiceOverlay.tsx`, `Spacebot/SpacebotContext.tsx` | Wing hero image, or drop and use `WingDriveLogo.svg` |
| `packages/interface/src/components/Orb.tsx` (WebGL orb, code not image) | Upstream animated orb | About, startup overlay, voice overlay | Decide: restyle (wing palette/shape) or replace with an animated wing |
| `packages/assets/images/BloomOne.png`, `BloomTwo.png`, `BloomThree.png`, `AlphaBg.png`, `AlphaBg_Light.png` | Upstream glow/background art | not referenced in code | Delete unless a new design wants them |
| `packages/assets/videos/WingIntro.mp4`, `WingMobIntro.mp4` | Upstream orb intro animation | not referenced in code | New wing intro, or delete |
| `packages/assets/videos/Fda.mp4` | macOS Full Disk Access walkthrough recorded in the upstream app | not referenced in code | Re-record in WingDrive when onboarding needs it, or delete |
| `docs/public/WingColumnView.webp`, `WingGridView.webp`, `WingMediaView.webp`, `WingSizeView.webp`, `WingSplatView.webp`, `WingVideoPlayer.webp` | Screenshots of the upstream UI | docs | New screenshots of WingDrive |

### P3: Upstream design system

| File(s) | Today | Deliver |
|---|---|---|
| `packages/assets/icons/*.png` (about 197 file-kind and device icons, `*_Light` variants included) | Upstream icon pack | Decide whether to keep (license permitting) or commission a WingDrive set; low urgency, not a logo |
| `packages/assets/svgs/*.svg` (alert, arrows, caret, macOS window controls, spinner) | Generic UI glyphs | Keep unless the design system changes |

Not brand assets (keep): `packages/assets/svgs/brands/*` and `packages/assets/images/{Dropbox,GoogleDrive,iCloud,Mega}.png` are third-party service logos; `packages/assets/svgs/ext/**` are file-extension glyphs; `adapters/*` icons belong to their integrations; `SD.png`/`SD_Light.png` are the SD-card device icon.

Already WingDrive: `apps/tauri/src-tauri/icons/**`, `apps/tauri/WingDrive.icon/`, `apps/mobile/icon.png`, `apps/mobile/ios/WingDrive/Images.xcassets/AppIcon.appiconset/`, `packages/assets/images/WingDriveLogo.svg`, `.github/logo.svg`, `docs/logo/wingdrive-logo.svg`.

## Acceptance Criteria

- [x] P1 assets replaced with wing artwork
- [x] Orb (`Orb.tsx`, `Ball*.png`) replaced or restyled in About, startup, and voice overlays
- [x] Unreferenced upstream images and videos deleted or replaced, and `packages/assets/*/index.ts` regenerated
- [x] Docs screenshots retaken from WingDrive
- [x] Decision recorded for the upstream file-kind icon pack

## Verification (2026-10-02)

Wing artwork replaces About, startup, voice, favicon and mobile launch assets. Asset indexes regenerated. Documentation screenshots captured from the running WingDrive UI; old video and Splat captures removed. Generic file-kind artwork retained with attribution and licensing in NOTICE.md and packaging/licenses.
