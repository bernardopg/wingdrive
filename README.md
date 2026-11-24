<p align="center">
  <img src=".github/logo.svg" alt="WingDrive" width="96" height="96" />
</p>

<h1 align="center">WingDrive</h1>

<p align="center">
  <strong>Every drive you own, in one fast file manager.</strong><br/>
  Local-first. Indexed. Yours.
</p>

<p align="center">
  <a href="https://github.com/bernardopg/wingdrive/releases/latest"><img src="https://img.shields.io/github/v/release/bernardopg/wingdrive?label=release&color=7c3aed" alt="Latest release"/></a>
  <a href="https://github.com/bernardopg/wingdrive/actions/workflows/ci.yml"><img src="https://github.com/bernardopg/wingdrive/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI"/></a>
  <a href="./COVERAGE.md"><img src="https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcoverage.json" alt="Coverage"/></a>
  <a href="https://github.com/bernardopg/wingdrive/actions/workflows/codeql.yml"><img src="https://github.com/bernardopg/wingdrive/actions/workflows/codeql.yml/badge.svg?branch=main" alt="CodeQL"/></a>
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-0f172a" alt="Apache-2.0"/></a>
</p>

<p align="center">
  <a href="https://aur.archlinux.org/packages/wingdrive-bin/"><img src="https://img.shields.io/aur/version/wingdrive-bin?logo=archlinux&logoColor=white&label=AUR&color=1793d1" alt="AUR"/></a>
  <img src="https://img.shields.io/badge/Linux-supported-2ea44f?logo=linux&logoColor=white" alt="Linux: supported"/>
  <img src="https://img.shields.io/badge/macOS-preview-d97706?logo=apple&logoColor=white" alt="macOS: preview"/>
  <img src="https://img.shields.io/badge/Windows-beta-d97706?logo=windows&logoColor=white" alt="Windows: beta"/>
  <img src="https://img.shields.io/badge/Android-beta-d97706?logo=android&logoColor=white" alt="Android: beta"/>
  <a href="https://github.com/sponsors/bernardopg"><img src="https://img.shields.io/badge/sponsor-%E2%99%A5-db2777?logo=githubsponsors&logoColor=white" alt="Sponsor"/></a>
</p>

<p align="center">
  <a href="#install">Install</a> ·
  <a href="#first-steps">First steps</a> ·
  <a href="#highlights">Highlights</a> ·
  <a href="#documentation">Docs</a> ·
  <a href="#roadmap">Roadmap</a> ·
  <a href="#support-the-project">Support</a> ·
  <a href="./README.pt.md">Português</a>
</p>

<p align="center">
  <img src="docs/public/WingGridView.webp" alt="WingDrive grid view with the Inspector, dark theme" width="900"/>
</p>

<p align="center">
  <a href="https://wingdrive.org/#film"><img src="media-kit/social/readme-film-poster.png" alt="Watch the 35-second WingDrive film" width="720"/></a><br/>
  <sub>Watch the 35-second film · <a href="website/public/video/wingdrive-film.mp4">MP4</a> · <a href="media-kit/">Media kit</a></sub>
</p>

WingDrive is a desktop file manager for Linux with a daemon that indexes your folders, external
drives and cloud volumes into one library. Files stay where they are; the index makes them fast to
browse, search and recognize across your own devices. No account, no telemetry, no central server.

> **Alpha.** Linux is the supported platform, including use as the default file manager.
> Windows and Android ship in beta. Android Explorer supports local copy, move, and trash.
> macOS on Apple silicon ships as a preview `.dmg`, signed ad hoc and not notarized yet.
> Milestone M3 focuses on desktop releases for Windows and macOS.

## Install

| Where | How |
|---|---|
| Arch Linux and derivatives | `yay -S wingdrive-bin` (stable), or `yay -S wingdrive-nightly` for every release including alpha, beta and rc |
| Any Linux (x86_64) | AppImage, `.deb` or Flatpak bundle from [Releases](https://github.com/bernardopg/wingdrive/releases/latest) |
| Windows 10 and 11 (x64), beta | `.exe` or `.msi` from [Releases](https://github.com/bernardopg/wingdrive/releases/latest). SmartScreen may warn when a build ships without Authenticode signing |
| macOS on Apple silicon, preview | `.dmg` from [Releases](https://github.com/bernardopg/wingdrive/releases/latest). Not notarized yet: run `xattr -dr com.apple.quarantine /Applications/WingDrive.app` once after copying the app |
| Android (ARM64), beta | APK from [Releases](https://github.com/bernardopg/wingdrive/releases/latest) |
| Headless server | `docker run -d -p 8080:8080 -v wingdrive-data:/data ghcr.io/bernardopg/wingdrive/server`, or the `wing-server` archive for Linux x86_64 from [Releases](https://github.com/bernardopg/wingdrive/releases/latest) |
| Command line | `wing` ships with every desktop package and in the `wing-server` archive; Windows also gets `wing-<version>-windows-x64.zip` with `wing.exe` and `wing-daemon.exe` |
| From source | `just setup && just dev-desktop` ([guide](./docs/readme/installation.md)) |

Every release lists one `SHA256SUMS` for all files: `sha256sum -c SHA256SUMS --ignore-missing`.

After installing, WingDrive can open folders for `xdg-open` and for "Show in folder" in other apps,
start with your session and stay in the tray.

## First steps

1. **Add your folders.** On the Overview, click **Add Storage**, then **Local Folder**. WingDrive
   indexes the folder in place; nothing is moved or copied.
2. **Browse and search.** Pick a view (grid, list, columns, media, size), press <kbd>Space</kbd> for
   Quick Preview and <kbd>F3</kbd> for two panes. <kbd>Ctrl</kbd>+<kbd>Z</kbd> undoes the last
   file operation.
3. **Connect the rest.** **Settings → Integrations** signs in to Google Drive and Google Photos;
   **Pair** links your other computers.

The [user guide on wingdrive.org](https://wingdrive.org/docs/) walks through each step with screenshots.

## Highlights

<table>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/public/WingListView.webp" alt="List view with Git emblems and the docked terminal"/>
      <p><strong>Work where your code lives.</strong> Git emblems on files and folders, expandable
      folders in the list, and a terminal docked under the view (F4) that follows you around.</p>
    </td>
    <td width="50%" valign="top">
      <img src="docs/public/WingSearch.webp" alt="Search with kind, extension, size and date filters"/>
      <p><strong>Find it in seconds.</strong> Search refines by kind, extension, size, date and hidden
      files, with counts over the whole result.</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/public/WingMediaView.webp" alt="Media view of a photo folder"/>
      <p><strong>See folders your way.</strong> Grid, list, columns, media and size views, sections by
      kind or date, and Quick Preview for images, video, audio, PDF, text and code.</p>
    </td>
    <td width="50%" valign="top">
      <img src="docs/public/WingContextMenu.webp" alt="Context menu with compress options"/>
      <p><strong>Everything a file manager should do.</strong> Copy To and Move To, compress and
      extract, templates, Dolphin service menus, permissions, SHA-256 checksums, and undo.</p>
    </td>
  </tr>
  <tr>
    <td width="50%" valign="top">
      <img src="docs/public/WingIntegrations.webp" alt="Settings, Integrations with Google accounts and cloud sync"/>
      <p><strong>Your clouds, like local folders.</strong> Sign in to Google Drive and Google Photos
      from Settings, with several accounts. Copy, move, edit in your own apps and keep folders in
      sync both ways.</p>
    </td>
    <td width="50%" valign="top">
      <img src="docs/public/WingOverview.webp" alt="Overview with storage, devices, locations and volumes"/>
      <p><strong>Every drive at a glance.</strong> The Overview sums up storage, indexed files,
      devices, locations and volumes, and is where you add storage, pair devices and set up sync.</p>
    </td>
  </tr>
</table>

**Also in the box**

- Every file gets a BLAKE3 content identity, so the same file on two devices is one file.
- Pair your own devices over Iroh/QUIC to sync the library and send files with Wingdrop.
- Google Drive and Google Photos with several accounts: browse, copy, edit in local apps, sync folders
  and upload photos. OneDrive with your own Microsoft app registration. S3, Dropbox, Azure Blob and
  GCS as volumes through OpenDAL.
- `wing` CLI and `wing-server` for scripts and headless machines, both on the same daemon.
- English and Brazilian Portuguese, and themes such as Catppuccin, Gruvbox, Dracula and Tokyo Night.

## Documentation

| Topic | Start here |
|---|---|
| User guide, with screenshots | [wingdrive.org/docs](https://wingdrive.org/docs/): [First run](https://wingdrive.org/docs/first-run/) · [Explorer](https://wingdrive.org/docs/explorer/) · [Cloud volumes](https://wingdrive.org/docs/cloud/) · [Devices](https://wingdrive.org/docs/devices/) |
| Install and first run | [Installation](./docs/readme/installation.md) · [Get started](./docs/overview/get-started.mdx) |
| Libraries and locations | [Manage libraries](./docs/overview/manage-libraries.mdx) · [Add locations](./docs/overview/add-index-locations.mdx) |
| Server and CLI | [Self-hosting](./docs/overview/self-hosting.mdx) · [Linux deployment](./docs/cli/linux-deployment.mdx) · [CLI](./docs/cli/overview.mdx) |
| Devices and sync | [Pairing](./docs/core/pairing.mdx) · [Library sync](./docs/core/library-sync.mdx) · [Cloud](./docs/core/cloud-integration.mdx) · [Integrations](./docs/core/integrations.mdx) |
| How it works | [Architecture](./docs/readme/architecture.md) · [Core architecture](./docs/core/architecture.mdx) · [Indexing](./docs/core/indexing.mdx) |
| Extensions | [Extension SDK](./docs/extensions/introduction.mdx) |
| Contributing | [CONTRIBUTING](./CONTRIBUTING.md) · [AGENTS](./AGENTS.md) · [Dev environment](./docs/core/development-environment.mdx) · [Testing](./docs/core/testing.mdx) |
| Project | [Changelog](./CHANGELOG.md) · [Coverage](./COVERAGE.md) · [Roadmap](./.tasks/ROADMAP.md) · [Security](./SECURITY.md) · [Credits](./CREDITS.md) |

## How it works

One `wing-daemon` owns all state. The desktop app (Tauri 2 + React 19), the `wing` CLI and
`wing-server` with its web interface are thin clients that speak JSON-RPC to it; the mobile app
embeds the same Rust core. Stack: Rust, Tokio, SQLite (SeaORM), Iroh/QUIC, BLAKE3, OpenDAL.

Privacy by design: indexing, thumbnails and search run on your machine, traffic between paired
devices is encrypted, and destructive operations are guarded (secure delete refuses copy-on-write
filesystems, copies refuse to write into their own source, system mounts are never ejected).

## Roadmap

| Milestone | What it brings |
|---|---|
| **M1** · done | A reliable desktop file manager on Linux, from a production bundle |
| **M2** · done | Default file manager integration on Linux and dual-pane Explorer (F3). Desktop coverage follow-up remains open |
| **M3** · now | Desktop releases for Windows and macOS. Linux and Windows already support signed updates, macOS ships a preview `.dmg`, and notarization and platform validation continue |
| **M4** | File sync between devices, public sharing links, Wingdrop to nearby devices, Tailscale |
| **M5** | Semantic search, WASM extensions, encryption at rest, Android beta improvements and iOS releases |

The live plan, sprint by sprint, is in [`.tasks/`](./.tasks/ROADMAP.md) (Portuguese).

## Support the project

WingDrive is built in the open by one developer. If it saves you time, consider
[sponsoring on GitHub](https://github.com/sponsors/bernardopg). Bug reports, ideas and pull
requests help just as much: [Issues](https://github.com/bernardopg/wingdrive/issues/new/choose) ·
[Discussions](https://github.com/bernardopg/wingdrive/discussions).

## Contact

- Website: [wingdrive.org](https://wingdrive.org)
- Discord: [WingDrive server](https://discord.gg/n8uwvrYMCh)

**Bernardo Gomes**, author and maintainer

- Email: [bernardo.gomes@bebitterbebetter.com.br](mailto:bernardo.gomes@bebitterbebetter.com.br)
- Personal site: [bebitterbebetter.com.br](https://bebitterbebetter.com.br)
- Instagram: [@be.pgomes](https://instagram.com/be.pgomes)

Report security issues privately through [GitHub Security Advisories](https://github.com/bernardopg/wingdrive/security/advisories/new) ([SECURITY.md](./SECURITY.md)), not in public issues.

## License

[Apache-2.0](./LICENSE). WingDrive started from the Spacedrive codebase; that origin and every
third-party notice are in [CREDITS.md](./CREDITS.md). File-kind icons in `packages/assets/icons` stay
under GPL-3.0-only.
