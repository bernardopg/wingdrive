# App Store Distribution

Status: plan, 2026-11-28. Epic `DIST-000` in `.tasks/core/`.

WingDrive ships today from GitHub Releases (AppImage, `.deb`, Flatpak bundle, Windows `.exe` and
`.msi`, macOS `.dmg`, Android `.apk` and `.aab`, server archives), AUR (`wingdrive-bin`,
`wingdrive-nightly`) and GHCR. This document lists the stores worth publishing in, what each one
requires, what in this repository blocks it, and the steps in order. Facts about store rules were
checked on 2026-11-28 against the sources at the end. Store rules change, so recheck a store's
page before you start its task.

## Recommended order

| # | Store | Platform | Cost | Blocker in this repo | Effort | Tasks |
|---|---|---|---|---|---|---|
| 1 | Obtainium | Android | Free | None | 1 hour | DIST-004 |
| 2 | Google Play | Android | USD 25 once | Signing key choice, All files access review, 14-day closed test | 3 to 5 weeks, mostly waiting | DIST-001, DIST-002, DIST-003, MOB-020 |
| 3 | winget | Windows | Free | None (unsigned works, signed is better) | 1 day | DIST-009 |
| 4 | Flathub | Linux | Free | App ID `com.wingdrive.desktop` is on a domain the project does not own | 2 to 4 days | DIST-008 |
| 5 | F-Droid | Android | Free | Play Services and ML Kit in `expo-camera`; heavy build | 2 to 4 weeks | MOB-018, MOB-020, DIST-006 |
| 6 | Homebrew tap | macOS | Free | Official cask needs notarization | Half a day (tap) | DIST-011 |
| 7 | Microsoft Store | Windows | Free account, paid certificate | Installers must be Authenticode-signed | 2 days after REL-002 | DIST-010 |
| 8 | Android developer verification | Android | Free with a Play account | Needed for sideloaded APKs in 2027 | 1 hour | DIST-005 |
| 9 | IzzyOnDroid | Android | Free | 30 MB size limit, policy against AI-assisted apps | Decision first | MOB-019, DIST-007 |
| 10 | Snap Store | Linux | Free | Strict confinement only | 2 days | DIST-012 |

Why this order:

- Google Play reaches the most Android users, and its closed test takes at least 14 days plus
  review, so start it first and do the other stores while you wait.
- Obtainium and winget cost almost nothing and work with the current release assets.
- Flathub reaches every Linux distribution that is not Arch or Debian based. The release already
  builds a Flatpak, so the work is the app ID and metadata.
- F-Droid matters to the privacy-minded users a local-first file manager attracts, but it needs code
  changes (MOB-018) and a from-source build recipe.
- Microsoft Store and the official Homebrew cask wait on paid signing (REL-002).

Not recommended now:

- **Mac App Store.** It requires the App Sandbox. A file manager with a background daemon, a CLI
  and access to the whole disk does not fit it without a large rework. The `.dmg` and Homebrew
  cover macOS.
- **Accrescent.** Submissions are closed to new developers, and it requires proof that you own the
  domain in the app ID (`wingdrive.com` for `com.wingdrive.app`), which the project does not.
- **Amazon Appstore.** Amazon stopped the Appstore on non-Fire Android devices in 2025.
- **Samsung Galaxy Store, Huawei AppGallery.** Small reach for this audience; revisit after Play.

## Cross-cutting decisions

### Android signing key

The GitHub Release APK is signed with the key in the `WINGDRIVE_ANDROID_KEYSTORE_*` repository
secrets (`apps/mobile/android/app/build.gradle`, `.github/workflows/mobile.yml`). Users who
installed it can only update to an APK signed with the same key.

Google Play re-signs every app with its app signing key. If you let Play generate a new key, the
Play build and the GitHub build cannot update each other, and a user who switches must uninstall
and lose local data. Avoid that: when you create the app in Play Console, choose "Use a different
key" and upload the current release key with Google's PEPK tool. Then generate a new upload key for
Play uploads. This choice is permanent. Play allows one key upgrade later, but it does not reach
users of other channels.

The same key should sign the F-Droid build (through reproducible builds, DIST-006) and is the key
you register for Android developer verification (DIST-005).

### Package names and domains

| Identifier | Where | Domain it implies | Problem |
|---|---|---|---|
| `com.wingdrive.app` | Android (`apps/mobile/app.json`) | wingdrive.com | Accrescent requires domain proof. Play, F-Droid and developer verification do not. |
| `com.wingdrive.desktop` | Tauri identifier and Flatpak app ID | wingdrive.com | Flathub requires control of the domain, so the Flatpak ID must change. |

`wingdrive.com` is a parked page owned by someone else; the project owns `wingdrive.org`.

- Keep `com.wingdrive.app` on Android. Changing it would orphan every installed APK, and the stores
  that matter accept it. If `wingdrive.com` is ever bought, nothing needs to change.
- Use `org.wingdrive.WingDrive` as the Flatpak app ID on Flathub (DIST-008). Keep the Tauri
  identifier `com.wingdrive.desktop`, because it names the config and data directories of every
  existing install.

### Android developer verification

Since 2026-09-30, certified Android devices in Brazil, Indonesia, Singapore and Thailand refuse
apps from unregistered developers when installed from participating stores. Google extends this
to all apps on certified devices, sideloaded ones included, in 2027. Play registers the apps it
distributes. For the GitHub APK, Obtainium and F-Droid, register `com.wingdrive.app` and the
SHA-256 fingerprint of every key that signs a published APK in Play Console > Android developer
verification (DIST-005). Unregistered apps still install through an "advanced flow" with a
one-time 24-hour wait, which most users will not go through.

### Listing content

Every store wants the same material. Make it once:

- Short and long descriptions in English and Brazilian Portuguese
- Icon 512x512, Play feature graphic 1024x500
- Phone screenshots (4 to 8), desktop screenshots at 1366x768 or larger (Flathub, Microsoft Store)
- Privacy policy URL https://wingdrive.org/privacy (live, INTG-002) with an Android section
- Support email and website https://wingdrive.org

Android text and images go in `fastlane/metadata/android/<locale>/` (MOB-020). F-Droid and
IzzyOnDroid read that folder from the repository; Play takes the same files through its API.

### Updater per channel

`apps/tauri/src-tauri/src/updates.rs` self-updates only the AppImage and the Windows installers.
Any Linux run without `$APPIMAGE` reports "package manager", so Flatpak and Snap installs never
download updates themselves. The hint text names `yay -Syu`; DIST-008 makes it name
`flatpak update` when `FLATPAK_ID` is set. On Windows, the Microsoft Store does not update `.exe`
and `.msi` apps, and winget only installs what the manifest points to, so the built-in updater stays
on for both. The Android app has no updater.

### Google sign-in client per channel

`core/src/service/integrations/mod.rs` reads the bundled Google and Microsoft clients at build time
with `option_env!`. Stores that rebuild from source (F-Droid, and Flathub if it ever builds from
source) do not have the secrets, so those builds ask the user for their own client. F-Droid does
not sign up for API keys on behalf of apps. The mobile build does not receive the secrets today
(`.github/workflows/mobile.yml`), so every Android channel behaves the same. Keep the
user-supplied client path working.

## Google Play

**Why:** most Android users only install from Play. Since 2026-09-30, Play apps are also the ones
that install without friction on certified devices in Brazil.

**Requirements (2026-11):**

- Personal Play Console account: USD 25 once, government ID, a verified Android device. An
  organization account needs a legal entity and a D-U-N-S number but skips the closed test rule.
- Personal accounts created after 2023-11-13 must run a closed test with at least 12 testers opted
  in for 14 consecutive days before applying for production. Internal testing does not count.
  Review of the application usually takes up to 7 days.
- New apps publish as Android App Bundles with Play App Signing.
- New apps and updates must target API level 36 since 2026-08-31.
- Store listing, privacy policy, Data safety form, content rating, target audience, ads
  declaration, app access instructions.
- `MANAGE_EXTERNAL_STORAGE` (All files access) needs the Permissions Declaration Form. File
  managers are a permitted use. Expect to send a demo video.

**State of this repository:**

| Item | State | Reference |
|---|---|---|
| Signed AAB on tags | Done: `WingDrive-3.8.2-android.aab`, 62 MB | `.github/workflows/mobile.yml`, MOB-009 |
| Upload key secrets | Present (`WINGDRIVE_ANDROID_KEYSTORE_*`) | REL-001 |
| Target SDK | 36, compile SDK 37 | `apps/mobile/app.json` |
| versionCode | Monotonic, derived from the tag | `scripts/release/android_version.py` |
| All files access | Requested | `apps/mobile/app.json`, `AndroidManifest.xml` |
| Camera | Used for the pairing QR code | `apps/mobile/src/components/PairingPanel.tsx` |
| Foreground services | None declared, so no type declaration is needed | `AndroidManifest.xml` |
| Accounts in the app | None, so the account deletion rule does not apply | Recheck when SHARE-011 adds accounts |
| Orientation | Locked to portrait; Android 16 ignores this on large screens for target 36 | `AndroidManifest.xml` |
| Listing assets | Missing | MOB-020 |

**Steps:**

1. Create the personal account and verify identity and device (DIST-001).
2. Create `com.wingdrive.app`. Upload the current release key as the app signing key with PEPK.
   Create a new upload key and keep it in Bitwarden.
3. Upload the AAB of the latest tag to the internal track, check it installs, then create a closed
   track.
4. Recruit at least 20 testers through Discord and friends. Keep 12 or more opted in for 14
   consecutive days. Ship fixes to the same track meanwhile.
5. While the test runs: write the listing (MOB-020), add the Android section to the privacy
   policy, fill Data safety, content rating and target audience, and send the All files access
   declaration with a video (DIST-002).
6. Apply for production and answer the closed test questions.
7. Release to production with a staged rollout.
8. Automate: a service account uploads each stable AAB to the internal track from
   `mobile.yml`; promotion to production stays manual (DIST-003).

Data safety answers, as the app works today: no data collected or shared by the developer; files
sent to Google Drive, Google Photos or OneDrive happen at the user's request, which Play does not
count as sharing; traffic is encrypted in transit; no account, so no deletion request flow.

**Effort:** 2 to 3 days of work spread over 3 to 5 weeks of waiting.

## F-Droid

**Why:** the main store for users who avoid Google services, and the audience of a local-first file
manager. It builds from source, which adds trust.

**Requirements:**

- FLOSS license (Apache-2.0 is fine) and a FLOSS toolchain.
- F-Droid builds the app itself. Prebuilt binaries are allowed only from Maven Central, Google
  Maven and a few others, and must be freely licensed. Rust from rustup, Node packages and Hermes
  are accepted.
- No Google Play Services, Firebase or proprietary analytics. Apps that cannot build without them
  are rejected.
- Anti-features label what remains: WingDrive will carry `NonFreeNet` for the Google Drive, Google
  Photos and OneDrive integrations.
- No self-update that bypasses F-Droid without opt-in. The Android app has none.

**Blockers in this repository:**

- `expo-camera` 58 adds `play-services-code-scanner`, `mlkit:barcode-scanning` and
  `play-services-mlkit-document-scanner` unless `expo.camera.barcode-scanner-enabled` and
  `expo.camera.document-scanner-enabled` are false in Gradle properties. The pairing QR scanner in
  `PairingPanel.tsx` uses the barcode scanner. MOB-018 replaces it with a FLOSS decoder.
- The build has many stages: Rust core through cargo-ndk (`xtask build-mobile`,
  `apps/mobile/modules/wing-mobile-core/android`), Bun install, Expo prebuild patches
  (`apps/mobile/patches`, `scripts/patch-svg.sh`), Gradle. It must run from a clean checkout with no
  secrets and no network beyond accepted repositories. Bun is not on F-Droid's list of accepted
  binaries; ask the reviewers, or use npm for the F-Droid build.
- F-Droid signs with its own key unless the build is reproducible. Then users cannot move between
  F-Droid and the GitHub or Play build. Reproducible Rust, Hermes and Gradle output takes work.

**Steps:**

1. MOB-018 and MOB-020.
2. Write one build script that works from a clean clone. Pin the NDK, Rust targets and cargo-ndk.
3. Test with `fdroidserver` locally: `fdroid build -v -l com.wingdrive.app`.
4. Write `metadata/com.wingdrive.app.yml` in a fork of gitlab.com/fdroid/fdroiddata: categories,
   license, `AntiFeatures: NonFreeNet`, one build entry per ABI, `UpdateCheckMode: Tags`,
   `AutoUpdateMode: Version`.
5. Open the merge request and answer the review. Expect several rounds.
6. Make the build reproducible and add `Binaries` and `AllowedAPKSigningKeys`, so F-Droid ships the
   APK signed with the project key.
7. Register the key F-Droid uses in Android developer verification (DIST-005).

**Effort:** 1 to 2 weeks of work, plus review time.

## IzzyOnDroid

**Why:** ships the developer's own APK from GitHub Releases, with updates within a day, inside the
F-Droid client.

**Blockers:**

- About 30 MB per app. The arm64 APK is 121 MB. MOB-019 measures and cuts it, but reaching 30 MB
  with the Rust core may not be possible.
- No proprietary libraries (MOB-018).
- The inclusion policy says IzzyOnDroid is "strongly opposed to apps which are fully or in part
  created by generative AI tools". This repository is developed with AI assistance (`AGENTS.md`,
  `.agents/skills`). Applying without saying so would be dishonest, and saying so will likely end
  the request.

**Steps:** decide first (DIST-007). If you apply: MOB-018, MOB-019, MOB-020, then an inclusion
request at codeberg.org/IzzyOnDroid/repodata.

## Obtainium

**Why:** installs and updates the GitHub Release APK directly, with no store review. It already
works.

**Steps (DIST-004):** add an "Add to Obtainium" link to the website download page and the README,
document the APK name filter, and keep asset names stable. Register the key for developer
verification (DIST-005) so these installs keep working in 2027.

**Effort:** 1 hour.

## Flathub

**Why:** the default app source on Fedora, Linux Mint, elementary OS, Pop!_OS and Steam Deck, and
available on every distribution.

**Requirements:**

- App ID on a domain the project controls, at least three components, matching the metainfo `<id>`.
  Flathub checks it by asking for a token at
  `https://<domain>/.well-known/org.flathub.VerifiedApps.txt`.
- Metainfo that passes `flatpak-builder-lint`: screenshots, releases, content rating, developer,
  launchable.
- Narrow permissions, or a linter exception with a reason.
- Building from source is preferred. Repackaging the project's own release binaries is accepted
  case by case.

**Blockers in this repository:**

- `packaging/flatpak/com.wingdrive.desktop.yml` and `com.wingdrive.desktop.metainfo.xml` use
  `com.wingdrive.desktop`. The project does not control `wingdrive.com`. Rename the Flatpak ID to
  `org.wingdrive.WingDrive`.
- The metainfo has no `<screenshots>` and no `<releases>`.
- `--filesystem=host` needs an exception. A file manager is the standard justification.
- The manifest reads a local `WingDrive.deb`. Flathub needs a URL with a checksum and
  `x-checker-data`, so the Flathub bot opens a pull request for each release.
- Only x86_64 is built. Mark `only-arches: [x86_64]` until an arm64 `.deb` exists.
- Repackaging the release `.deb` keeps the bundled Google client. A from-source build on Flathub
  would not have it.

**Steps (DIST-008):**

1. Rename the app ID in the manifest, metainfo, desktop file, icons and `flatpak-bundle.yml`.
2. Complete the metainfo and pass the linter.
3. Point the manifest at the release `.deb` by URL and sha256, add `x-checker-data`.
4. Fork github.com/flathub/flathub, branch from `new-pr`, add the manifest, open the pull request
   and explain `--filesystem=host`.
5. After acceptance, publish the verification token in
   `website/public/.well-known/org.flathub.VerifiedApps.txt`.

**Effort:** 2 to 4 days, plus review.

## winget

**Why:** the package manager built into Windows 10 and 11, used by `winget install` and by tools
like UniGetUI.

**Requirements:** a silent installer at a direct URL (the NSIS `-x64-setup.exe` installs with
`/S`), a manifest pull request to microsoft/winget-pkgs, the CLA. No account fee. Signing is not
required, but Microsoft's validation scans the installer, and unsigned installers are more likely to
need manual review.

**Steps (DIST-009):**

1. `wingetcreate new` with the stable installer URL; package ID `WingDrive.WingDrive`.
2. Sign the CLA and answer validation comments.
3. Add a release job on `windows-latest` that runs
   `wingetcreate update WingDrive.WingDrive --version <v> --urls <url> --submit` on stable tags,
   with a classic PAT in the `WINGET_TOKEN` secret.

**Effort:** 1 day.

## Microsoft Store

**Why:** the store built into Windows, trusted by users who never use a terminal.

**Requirements:**

- Individual developer accounts are free.
- An `.exe` or `.msi` submission must be signed, with every PE file inside it, by a certificate from
  a CA in the Microsoft Trusted Root Program. The Store does not re-sign it. The URL must be
  versioned and the file behind it must not change.
- MSIX packages get signed by the Store for free, but WingDrive would need MSIX packaging of the
  daemon, tray, run-at-logon and the UAC helper of TAURI-017.

**Blocker:** REL-002. The REL-001 plan counts on Azure Artifact Signing, which is open to
organizations in the USA, Canada, EU and UK and to individuals in the USA and Canada only. An
individual in Brazil needs an OV code signing certificate from a CA such as Certum, Sectigo or
SSL.com (typically USD 150 to 300 a year; Certum offers cheaper certificates for open source
developers) on a hardware token or the CA's cloud signing service.

**Steps (DIST-010):**

1. Get the certificate and sign the installers in `windows-bundle.yml` (REL-002).
2. Register at storedeveloper.microsoft.com, reserve the name WingDrive.
3. Create an MSI/EXE app with the signed installer URL from the GitHub Release and the silent
   switch.
4. Listing, screenshots, privacy policy, IARC rating, submit.
5. For each release, update the installer URL, by hand or through the submission API.

**Effort:** 2 days after REL-002.

## Homebrew

**Why:** how most developers on macOS install apps.

**Requirements:** since 2026-09-01 homebrew/cask disables casks that fail Gatekeeper. The `.dmg`
is ad hoc signed and not notarized (`macos-bundle.yml`), so the official cask waits on a Developer
ID and notarization (REL-002, USD 99 a year). homebrew/cask also has notability rules, stricter for
self-submitted apps. A tap owned by the project has neither rule.

**Steps (DIST-011):** create `bernardopg/homebrew-wingdrive` with a cask for the arm64 `.dmg`,
update it from the release workflow, and submit to homebrew/cask after notarization.

**Effort:** half a day for the tap.

## Snap Store

**Why:** the default store on Ubuntu.

**Requirements:** a free Snapcraft account and a registered name. Reviewers refuse classic
confinement and `removable-media` auto-connect for file managers, so the snap is strictly confined
with `home` and a manually connected `removable-media`.

**Steps (DIST-012):** `snap/snapcraft.yaml` that repackages the `.deb`, a check with
`snapctl is-connected` that tells the user to connect `removable-media`, and publishing to the edge
channel from the release workflow.

**Effort:** 2 days. Lowest priority: the `.deb`, AppImage and Flathub already cover Ubuntu.

## Combined checklist

Accounts and money:

- [ ] Google Play Console personal account, USD 25 (DIST-001)
- [ ] Microsoft Partner Center individual account, free (DIST-010)
- [ ] Code signing certificate for Windows (REL-002)
- [ ] Apple Developer Program, USD 99 a year (REL-002, for the official Homebrew cask)
- [ ] Snapcraft account, free (DIST-012)

Code and build:

- [ ] Pairing QR scanner without Play Services or ML Kit (MOB-018)
- [ ] Smaller APK, per-ABI builds (MOB-019)
- [ ] Flatpak app ID `org.wingdrive.WingDrive`, complete metainfo (DIST-008)
- [ ] Updater hint for Flatpak (DIST-008)
- [ ] Clean-checkout Android build for F-Droid, then reproducible (DIST-006)

Content:

- [ ] Fastlane metadata and screenshots in en-US and pt-BR (MOB-020)
- [ ] Desktop screenshots for Flathub and the Microsoft Store
- [ ] Android section in the privacy policy (DIST-002)
- [ ] Demo video for the All files access declaration (DIST-002)

Signing:

- [ ] Current Android release key uploaded as the Play app signing key; new upload key (DIST-001)
- [ ] Package name and keys registered for Android developer verification (DIST-005)

Automation:

- [ ] AAB to the Play internal track (DIST-003)
- [ ] winget pull request on stable tags (DIST-009)
- [ ] Flathub bot through `x-checker-data` (DIST-008)
- [ ] Homebrew tap update (DIST-011)
- [ ] Store links on the website download page and README when each listing goes live

## Sources

- Google Play target API level: https://support.google.com/googleplay/android-developer/answer/11926878
- Google Play testing requirements for new personal accounts: https://support.google.com/googleplay/android-developer/answer/14151465
- Google Play All files access: https://support.google.com/googleplay/android-developer/answer/10467955
- Android developer verification: https://developer.android.com/developer-verification
- Registering open source apps: https://developer.android.com/developer-verification/guides/open-source-app-registration
- F-Droid inclusion policy: https://f-droid.org/docs/Inclusion_Policy/
- IzzyOnDroid inclusion policy: https://izzyondroid.org/docs/general/AppInclusionPolicy/
- Accrescent publishing requirements: https://accrescent.app/docs/guide/appendix/requirements/
- Flathub requirements: https://docs.flathub.org/docs/for-app-authors/requirements
- Flathub linter: https://docs.flathub.org/docs/for-app-authors/linter
- Microsoft Store MSI/EXE requirements: https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msi/app-package-requirements
- Windows code signing options: https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options
- winget manifests: https://learn.microsoft.com/en-us/windows/package-manager/package/manifest
- Snap confinement discussion for Tauri file managers: https://forum.snapcraft.io/t/request-for-classic-confinement-for-aeroftp/49921
- Homebrew Gatekeeper rule: https://github.com/orgs/Homebrew/discussions/6482
