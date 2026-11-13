# Changelog

All notable changes to WingDrive are recorded here. Versions follow the product
version in the workspace manifests and the matching `v*` git tag.

## 3.2.0 — 2026-11-13

Google Drive and Google Photos with several accounts per library, file
operations and editing on cloud volumes, two-way cloud sync, desktop shell
coverage at 91.6% with the bugs it found fixed, and the `sdPath` to `wdPath`
rename (#383 to #388).

### Added

- Google integration (CLOUD-004). Connect several Google accounts per library
  through the browser (Settings > Integrations, `wing integrations connect
  google`). Each Google Drive shows up as a volume and a sidebar location you
  browse live, with Google Docs, Sheets and Slides as `.docx`, `.xlsx` and
  `.pptx` exports. Grants are stored encrypted with the library key and
  disconnecting revokes them at Google. See `docs/core/integrations.mdx`
  (#383).
- Copy, move, rename, new folder and move to trash work on cloud volumes, in
  both directions between local folders and the cloud (FILE-003).
- Double-clicking a Drive file opens a local copy whose saves upload back;
  if the file changed in the cloud meanwhile, the edit is kept as a conflict
  copy. Google Docs open in the browser.
- Cloud sync keeps a local folder and a cloud folder in step, both ways or
  one way, on demand or every N minutes (`wing integrations cloud-sync`).
- Google Photos: upload photos and videos from the context menu, optionally
  into an album, and import photos picked on Google's page.
- Google Drive in the Explorer: Quick Look downloads the file to a cache,
  Open in Browser opens the item on drive.google.com, New File and templates
  work in Drive folders, files dropped from the desktop upload, Copy To
  downloads, the Column view and drag and drop understand Drive folders, the
  Inspector shows the full `gdrive://` path, and the Drive volume in the
  sidebar opens the Drive (#385).

### Changed

- Adding Google Drive no longer asks for pasted OAuth tokens; it signs in
  through the browser.
- Code and task tracker: `sdPath` is now `wdPath` in the core, interface and
  clients, and every open task has an owner, a milestone and a sprint from
  S05 to S14 (BRAND-003).

### Fixed

- Cloud volumes disappeared 30 seconds after startup, when the volume
  monitor's refresh did not find them among local disks.
- Cloud locations were listed as local paths, so opening one showed nothing.
- Extracting a TAR whose hard link pointed at a symbolic link already in the
  destination copied the link's external target into the extraction. Hard
  links to anything but a regular file are now rejected and opened without
  following links.
- The archive browse cache never removed old plain files at its root, so
  they stayed on disk and logged a warning on every prune.
- Deleting an item in a cloud folder did nothing: the delete job sent cloud
  paths to the strategy for other devices, which skipped them.
- Undoing a move to trash failed with "no longer in the trash" when the item
  was trashed through a folder reached by a symbolic link (#386).
- Dragging on Linux and Windows left a drag session behind, so every new
  drag failed for 30 seconds, and the frontend could not cancel or recognize
  the end of a drag on macOS (#387).
- Installing the daemon as a systemd service broke data folder paths with
  spaces, `%` or `$`; every `ExecStart` argument is now quoted (#387).
- Location exports made before the rename to WingDrive were rejected on
  import (#387).

### Tests

- Boundary cases for tag validation, rename validation and archive formats
  and extraction.
- Desktop shell line coverage from 80.5% to 91.6%: Tauri commands, daemon
  lifecycle, file chooser portal, drag, clipboard, services menu and the
  media server (COV-07, #386, #387).
- Core integration tests pass `clippy --all-targets` and the tests that set
  the process-wide device ID run one at a time in CI (#385, #387).

### Planned

- Integrations roadmap (`INTG-000`): bundled Google OAuth client, Google app
  verification, privacy policy and terms, reconnect flow and real-Google
  validation, so any user connects with a sign-in; then Dropbox, OneDrive,
  Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA, Proton Drive, object
  storage, SFTP and SMB. See `.tasks/ROADMAP.md`.

## 3.1.1 — 2026-11-12

WingDrive as the Linux file chooser for browsers, Flatpak and GTK4 apps, undo
for rename and new items through the core journal, a fix for files with an
uppercase extension, about 175 new tests and routine dependency updates
(#368 to #380).

### Added

- WingDrive can be the file chooser for uploads and saves in browsers, Flatpak
  and GTK4 apps on Linux, through an `org.freedesktop.impl.portal.FileChooser`
  backend (Settings > General > File chooser). The switch keeps the previous
  portal value and restores it when turned off. The `.deb` and AUR packages
  install `wingdrive.portal` and a D-Bus activation file for it (DESK-051, #375).

### Changed

- Ctrl+Z undoes rename, batch rename, new folder, new file, new file from
  template and new link through the core journal, in the desktop app and the
  web UI, and `wing file undo` and `wing file history` include them. The
  Tauri-side undo is gone. Undo never overwrites an item that took the old
  name, including a dangling link (FILE-008, #378).
- Copy To, Move To and Extract To open the folder picker in the folder of the
  selection (DESK-090, #379).

### Fixed

- Files with an uppercase extension, such as camera `.JPG` photos, in indexed
  locations could not be opened, copied, deleted or hashed. Rescan a location
  to repair files indexed by an earlier version. The `.jpg` search filter also
  matches `.JPG` files (DESK-096, #379).
- The status bar shows free space for folders opened from outside the app and
  on Windows (DESK-094, DESK-095, #379).
- "This Location" search on Windows searched the whole library (DESK-093, #379).
- The SHA-256 check in the Inspector accepts BSD-style, `openssl` and
  `certutil` checksum lines (DESK-092, #379).
- Select by Pattern reads `[!]` and `[]]` the way the shell does, and an
  invalid range such as `[z-a]` matches literally instead of failing, also in
  the folder filter (DESK-060, DESK-091, #374, #379).
- Copy and move conflict planning no longer treats a destination it cannot
  read (permission denied) as free (CORE-020, #376).
- `wing logs --job-id` filters the log history too, not only new lines
  (CLI-010, #377).
- wingdrive.org deploys again: the site data folder was never committed. The
  Discord invite on the site, the READMEs and the issue template is now
  permanent (#373).

### Tests

- About 175 new tests across the core, the CLI and the desktop interface
  (#374 to #379).

### Dependencies

- Rust: `jsonschema` 0.61.0, `sevenz-rust2` 0.25.0, `notify` 8.2.2, `zip`
  9.4.2, `uuid` 1.29.1, `toml` 1.2.1 and `smallvec` 1.16.5 (#370 to #372).
- Interface: TanStack Query 5.107.1, `framer-motion` 14.10.1, `zustand` 5.1.0,
  `maplibre-gl` 6.17.0, Tailwind CSS 4.3.5 and minor tooling updates (#369).
- CI: `taiki-e/install-action` 2.90.3 (#368).

## 3.1.0 — 2026-11-10

Undo for copy, overwrite, move and trash, copies that never destroy data when
they are cancelled or fail, paired devices that keep each other's name current
and forget each other together, and a bundled SQLite with the CVE-2026-119375
fix. The test suites grew by 79 cases and found seven more bugs (#347 to #360).

### Added

- Undo for copy, overwrite, cross-volume move and trash. File jobs journal what
  they changed, `wing file undo` and Ctrl+Z in the desktop app reverse it, and
  `wing file history` lists what can still be undone. A file that a copy
  replaces is kept for 7 days in a hidden `.wing-undo-*` folder next to it.
  Undo skips anything that changed since the operation. The journal lives in
  each library's job database and is never synced. RPC clients get the
  `files.undo` action and the `files.undo_history` query (FILE-008).
- Paired devices send their current name, type, operating system and app
  version every time they connect, so a renamed or reinstalled device shows up
  with its new description on the other side (MOB-013).
- Removing a paired device (`wing network revoke`, `network.device.revoke`)
  tells the other device, which drops the pairing too. The output reports
  `peer_notified`. A device that is offline, or still runs 3.0.x, keeps the
  pairing until it is removed there; the CLI says so.

### Changed

- Mobile: Expo 58.0.23 with the `react-native-safe-area-context` version its
  patch needs, and NativeWind 5.0.0-rc.1 with its `react-native-css`
  3.1.0-rc.1 peer. The AssetRegistry patch moves to the new version.
- Dependabot updates for the Cargo, bun and GitHub Actions groups (#349, #350,
  #352).

### Fixed

- Cancelling a copy that replaces an existing file no longer destroys that
  file. Copies are written under a temporary name and renamed into place, so a
  cancelled or failed copy also leaves no truncated file behind.
- A cancelled or paused copy stops as cancelled or paused. It used to count the
  current item as failed and carry on with the rest of the batch.
- Copies keep the modification time of files and folders. The
  `preserve_timestamps` option existed and defaulted to on, but nothing read it.
- A selected symbolic link is copied, moved and renamed as a link. A dangling
  one failed the whole operation, and a link to a folder copied the folder.
- Moving a folder that holds a pipe, socket or device file fails instead of
  skipping the item in silence and then deleting it with the source folder.
- `wing library sync-setup setup --action` takes and documents the same values:
  `share-local`, `join-remote` and `merge`. The help listed `register-only` and
  `create-shared`, which the parser refused (MOB-014).
- `wing library sync-setup` works with `--instance`. It asks the daemon for the
  local device instead of reading `device.json` from the default data folder,
  so it no longer demands `--local-device` for a named instance.
- An application whose desktop entry has `Exec=viewer %%f %%u` receives the
  selected files. The escaped `%%` was read as a file code and the files were
  dropped.
- The local media server answers 400 for a folder on Windows too. Windows
  refuses to open a folder as a file, and the error became a 403.
- Sidecar addresses round-trip: `WingPath::from_uri` accepts the `thumbstrips`
  and `gaussian_splats` URIs that `to_uri` produces, the MessagePack sidecar
  format reads back its own JSON (`message_pack`), and `cloud://` paths of a
  service without its own type (`CloudServiceType::Other`) round-trip.
- The indexing rule that rejects a folder holding certain subfolders
  (`RejectIfChildrenDirectoriesArePresent`) is applied. The ruler always saw
  the path as a file and skipped it. A folder the ruler cannot read reports the
  read error instead of a MessagePack decode error.

### Security

- Bundled SQLite moves to the version in `libsqlite3-sys` 0.39.1, which
  addresses RUSTSEC-2026-0459 and carries the fix for CVE-2026-119375, with
  `rusqlite` 0.41.1. SQLx 0.9.0 caps
  `libsqlite3-sys` below 0.38, so `patches/sqlx-sqlite-0.9.0` widens the bound
  until a SQLx release does. The patch no longer uses the `time` format APIs
  deprecated in 0.3.48. A core test fails if SQLite is older than 3.54.0.

### CI

- CI and release builds no longer print the "will be rejected by a future
  version of Rust" warning for `proc-macro-error2` 2.0.1 (pulled in by
  `wasmer-derive`): `patches/proc-macro-error2-2.0.1` carries the one-line
  E0365 fix until upstream releases it.
- The Windows installer job no longer warns that `ilammy/setup-nasm` runs on
  Node.js 20: `setup-system` downloads the same NASM 2.16.01 build itself and
  checks its SHA-256. macOS installs NASM with Homebrew.
- `setup-rust` installs the pinned toolchain before `rustup target add` and
  makes it the default, which clears the rustup deprecation warnings.
- The Rust CI jobs set `WING_SKIP_WEB_BUILD`, so `wing-server`'s build script
  no longer warns that bun is missing. An explicit skip no longer warns.
- Core integration tests: the unused `size_mb` warning on Windows is gone, and
  test tracing goes through the test writer, so passing tests no longer flood
  the CI log with daemon `WARN`/`DEBUG` lines.

## 3.0.1 — 2026-11-06

Bug fixes found by the expanded test suites and by running every fix in a
real daemon and the desktop app, and the Wingbot name in code and settings.

### Added

- The `wingdrive-nightly` AUR package receives every release, alpha, beta and
  rc included, and stable releases too. `wingdrive-bin` now takes stable
  releases only: before, a pre-release tag would have reached every
  `wingdrive-bin` user. The release workflow publishes both packages.

### Changed

- The README, the installation guide and the documentation introduction list
  the Windows installers as a beta, unsigned and not yet tested by hand on a
  real machine. The introduction no longer claims macOS support: macOS builds
  from source until its signed release.
- `TypeScript (Windows)` and `Rust Workspace (Windows)` are required checks on
  `main`, and the Windows Rust job fails the run again.
- `docs/core/cli.mdx` examples call `wing` instead of `sd`.
- `wing update` reports the newest release and prints how to install it for
  the way `wing` was installed: the AUR helper command, the `.deb` or AppImage
  link, the Windows installer, or the tag for a source build. It used to look
  for standalone binaries that releases never publish, and always failed with
  "Could not find WingDrive CLI for platform". It no longer downloads or
  replaces binaries, and `--force` is gone. The `beta` and `nightly` channels
  include pre-releases (CLI-002).

- The Wingbot settings use the `wingbot` key and `wingbot_*` fields in
  `config.app.update`. Settings saved under the old `spacebot` key are read
  and saved back under `wingbot`; saved tabs and windows still open Wingbot.
- Renaming refuses a name that is already taken before it starts, instead of
  failing inside the job. The name limit is 255 bytes on Linux and macOS and
  255 UTF-16 units on Windows, where accented names were refused too early.
- Linux CI runs every core unit test and the new file-operation, tag and
  config suites; core line coverage went from 30% to 44% and the desktop
  shell from 55% to 61% in a local run.

### Fixed

- Copying or moving a folder recreates the symbolic links inside it. It used
  to follow them: a link to the folder itself nested copies of it, links to
  other folders copied data from outside the selection, and dangling links
  vanished or failed the whole copy. Progress no longer walks the disk for a
  link to `/`.
- Deleting a dangling link, permanently or to the trash, works.
- A copy job no longer overwrites a file that appeared after the conflict
  question; that item fails with "Destination already exists". Renaming onto
  an existing folder no longer moves the item inside it.
- Copies with checksum verification read the destination back, so a bad write
  is detected. Before, the check compared the source with itself.
- "Keep both" keeps file name bytes that are not UTF-8.
- Creating a tag records this device as its creator, not the library. Tag
  search finds a tag by its alias alone. Tag and Wings colours must be
  `#RRGGBB` hex, and tag name limits count characters, not bytes.
- `WingPath::file_name` no longer leaks memory on every sidecar call, and
  joining cloud paths no longer doubles the `/`.
- Changing the theme or log level only in letter case is not a change, and an
  empty Wingbot URL (`/`) is refused.
- "Show in folder" selects the file on Linux. The D-Bus call was malformed and
  always fell back to opening the parent folder, which could also hang until
  the opened program closed. Names with `%`, `#` or `?` reach the file manager
  intact.
- Thumbnails and sidecar paths work for named instances and custom data
  folders.
- "Start at login" works when the app's path contains `%`, `"`, `$` or a
  backtick.
- Two named instances whose names differ only in punctuation or accents no
  longer share one D-Bus name and steal each other's windows.
- The local media server no longer advertises bytes for empty files or
  inverted ranges; sidecar requests with `..` are refused.
- GVFS mount names and `file://` launch URIs no longer decode incomplete
  escapes such as `%A` or `%+A`.
- Network discovery reads `Host=name` entries in `~/.ssh/config`.

## 3.0.0 — 2026-11-05

The first release with Windows installers (`.exe` and `.msi`), next to the
Linux packages. On Windows, CI tests each installer by installing it,
starting the app and uninstalling it. Nobody has run them by hand on a real
machine yet. They are unsigned, so SmartScreen warns about them (TAURI-016).

### Added

- Desktop notifications when a copy, move or other file job ends while
  WingDrive is in the background. They carry WingDrive's name, desktop entry
  and logo on every Linux notification server (GNOME Shell, Plasma,
  xfce4-notifyd, mako, dunst, swaync, Quickshell shells), use the
  notification portal under Flatpak, and use a toast under the installer's
  AppUserModelID on Windows.
- Linux packages install the WingDrive icon at 16, 48, 256 and 512 px and as
  a scalable SVG, so docks and notification servers no longer scale a 32 px
  icon.

- Windows releases: `windows-bundle.yml` builds an NSIS `.exe` and an `.msi`,
  installs each one, runs the daemon and the app from the installed files, and
  uninstalls them. The Release workflow publishes them, and any branch can
  produce them as run artifacts. Alpha, beta and rc tags ship the `.exe`
  only, because WiX accepts only numeric pre-release versions. The installers
  are not signed yet, so Windows SmartScreen warns about them.
- CI runs the TypeScript and Rust gates on Windows, as separate jobs.
- "Start at login" works on Windows through the current user's Run registry key.
- Copy, cut and paste of files work between WingDrive and Explorer on Windows,
  through the system file clipboard.
- `just dev-desktop` works on Windows through `scripts/dev-isolated.ps1`.

### Fixed

- WingDrive no longer takes `org.freedesktop.FileManager1` away from the
  system file manager: it claims the name only when it is the default handler
  for `inode/directory` (or `WINGDRIVE_FILEMANAGER1=1`). The AUR package no
  longer installs `com.wingdrive.FileManager1.service`, which dbus-broker
  reported as "not named after the D-Bus name" and which made it ignore
  Dolphin's activation file as a duplicate.
- The window's WM_CLASS and Wayland app_id are always `WingDrive`, matching
  `StartupWMClass`. Launching through `/usr/bin/wingdrive` used to produce
  `Wingdrive`, so docks showed an unknown app.

- A library no longer stays locked for up to an hour after a crash on Windows
  in languages where the `tasklist` "no tasks" message wraps onto two lines,
  such as Portuguese. The lock check now asks the kernel.
- Upgrading a secrets database from the old format no longer fails on Windows
  with os error 33.
- The operating system version is no longer empty on localized Windows.
- Search, media and tag results no longer mix `\` and `/` in Windows paths.
- `tauri dev` no longer fails with os error 32 on Windows when the daemon is
  running.
- The desktop app stops its daemon cleanly on Windows instead of killing it.
- Opening a `file:///C:/...` URI on Windows no longer fails.
- `wing` and `wing-daemon` no longer create an empty production data directory
  when `--data-dir` is given.
- An automatic backfill no longer overwrites a sync state that moved to Ready,
  catching up or paused while the backfill peer was chosen.
- The sync log (`sync.db`) and the job database use WAL with
  `synchronous=NORMAL`, like the library database. They used SQLite's
  defaults, which flushed and recreated a journal on every commit: one sync
  test made 2930 `fsync` calls, now 131. On Windows that flushing made each
  synced record take hundreds of milliseconds.
- On Windows, a named instance (`WINGDRIVE_INSTANCE`) no longer hands its launch
  to the everyday WingDrive and exits.
- "Retry as administrator" no longer appears on Windows and macOS, where it
  always failed. It stays on Linux.

### Security

- Apache Arrow moves to 60.1.0, which fixes out-of-bounds reads on malformed
  run-end encoded arrays (RUSTSEC-2026-0442).
- `task-validator` and `wing-bench` read YAML with `serde-saphyr` instead of
  the deprecated `serde_yaml`, which removes the unmaintained `unsafe-libyaml`
  (RUSTSEC-2024-0449). A test parses every shipped benchmark recipe.
- `osv-scanner.toml` records why each remaining advisory has no fix this
  repository can reach, and each entry expires on 2027-02-05 for review.

## 2.4.1 — 2026-11-03

Fixes the idle memory growth that failed the 2.4.0 release memory check, and
makes that check measure idle peaks instead of one snapshot.

### Changed

- README rewritten as a short landing page and documentation index, with dark
  screenshots of 2.4.0, platform and coverage badges, roadmap, contact and
  sponsorship links (Portuguese version too). The Discord links pointed to a
  server WingDrive does not have; help now goes to GitHub Discussions.
- A Coverage workflow measures Rust (`cargo-llvm-cov`) and frontend (`bun
  --coverage`) line coverage on every push to `main`. It publishes the README
  badge, per-component badges in `COVERAGE.md` and an HTML report on GitHub
  Pages, so no number is edited by hand.
- The release smoke test measures desktop memory as the peak over 30 s of idle
  time in fresh sessions, and the 550 MiB budget holds when most of three
  sessions stay under it. One snapshot taken at a random point of the garbage
  collection cycle failed the 2.4.0 release by chance. The release workflow runs
  the smoke test from the workflow commit, so a dispatch from `main` can
  re-verify an existing tag with the current gate.

### Fixed

- The desktop no longer re-renders the whole window and every open tab every
  3 s while connected. The daemon status poll replaced its state with an equal
  new object, so an idle app made garbage that WebKit collected only every
  minute or so, between about 320 and 470 MiB with 15 tabs. The renderer now
  stays flat while idle, at about 270 to 330 MiB on the same check (DESK-043).

## 2.4.0 — 2026-11-02

WingDrive is now an independent project: the inherited fork tracking is gone and
third-party credits live in `CREDITS.md` (#310). The M2 file manager features
from 2.3.0 were checked on the packaged Linux build, and that check found and
fixed two Explorer bugs.

### Added

- The docked terminal has a header with a hide button. Hiding it keeps the
  shell session, and it no longer reopens by itself on startup (#302).
- Sidebar groups, the entries inside them and the folder tree can be
  reordered by pointer or keyboard, with screen reader announcements. The
  order of dynamic entries and the folder tree position stay on this device
  (#302).

### Fixed

- After typing a path with Ctrl+L and Enter, keyboard focus no longer stays in
  the path bar, so Explorer shortcuts work right away (DESK-039).
- Context menus taller than the window scroll instead of losing their first
  items (DESK-040).
- Closing a terminal kills a shell that ignores the hangup, so no shell is left
  running after its tab or window closes (#311).

- Grouped views sort their sections in the active direction, so
  folders-first sorting no longer scrambles name sections (#302).
- Shift+arrow keys extend the selection in grouped grid views instead of
  replacing it (#302).
- Undo of "New File from Template" removes the copy only while it still
  matches the template's content and the entry it created; an edited or
  replaced file is kept (#302).

### Changed

- WingDrive identity is final: repository, packages and docs no longer present
  the project as a fork, and `CREDITS.md` and `NOTICE` ship in the desktop and
  AUR packages (#310).
- The M2 features from 2.3.0 (Copy To and Move To, invert and pattern
  selection, checksums, search filters, folder tree, groups, templates, service
  menus, Git emblems, docked terminal, new windows) passed the runtime check on
  the packaged Linux build.

- Dependency updates: Rust crates within ranges (zbus 6.2, uuid 1.29),
  resvg 0.49, sysinfo 0.40, Bun packages, GitHub Actions, and Expo SDK 58
  patch releases on mobile (react-native 0.88.1).
- Documentation: every page is reachable from the docs navigation, dated
  audits moved to `docs/design/history/`, and the legacy `mint.json` and the
  root `TODO` queue were removed (`.tasks/` is the backlog).

## 2.3.0 — 2026-10-29

Quick Preview that reads every common file type, a security fix for it, the
six M2 file manager features planned in S04 (#295 to #298), and a terminal
docked under the Explorer (#299 and this release). Each DESK task stays In
Progress until it is checked on the Linux desktop build.

### Added

- F4, or "Show Terminal Panel" in the empty-space menu, opens your shell in a
  panel under the Explorer. Navigating sends the shell along with `cd`, and
  `cd` in the shell moves the Explorer. A busy shell gets the `cd` once its
  program ends. The shell survives tab switches and hiding the panel
  (DESK-034).

- Quick Preview shows PDFs (pdf.js, page by page, with zoom), text, code and
  config files, including extensionless text files up to 2 MB (#295).
- Git emblems in grid and list views: `U` untracked, `A` added, `M` modified,
  `C` conflicted; a folder shows the strongest state inside it (DESK-035).
- "Show in Groups" splits grid and list views into sections by the sort key:
  first letter, date buckets, size buckets, kind or first tag (DESK-030).
- "New from Template" lists the files in the templates folder
  (`XDG_TEMPLATES_DIR`, else `~/Templates`) and copies the chosen one, keeping
  its permissions (DESK-031).
- Dolphin service menus appear in the file context menu; a menu in your own
  data folder must be executable to count (DESK-032).
- A "Folders" tree in the sidebar that follows the open folder, and folders
  that expand in place in list view, with a chevron or Right and Left
  (DESK-029).
- "Retry as administrator" after a permission error on delete, rename, new
  folder, copy or move, through polkit (DESK-033).

### Fixed

- Right after launch, the first folder could list "0 files": every collapsed
  row of the sidebar folder tree held a daemon connection and used up its limit
  of 100. Disabled queries no longer subscribe (DESK-037).
- Rows inside folders expanded in list view now show their Git emblem, refresh
  when files change, and are left out of the status bar and filter counts.
- Text and code previews were blank: the CSP blocked the asset protocol for
  `fetch`. Videos now fit the preview area. The Inspector's scrollbar no longer
  covers right-aligned values (#295).

### Security

- `pdfjs-dist` 6.4: versions 5.6.83 to 6.2.107 could run JavaScript from a
  malicious PDF (Dependabot alert 47). Standard fonts and image decoders are
  bundled, so PDFs that rely on non-embedded fonts keep their glyphs (#297).
- Dependency audit: `fast-uri`, `node-forge`, `seroval` and
  `postcss-selector-parser` updated past their advisories. `cargo audit`
  exceptions with no reachable fix (rsa, and ruzstd and memmap2 through the
  pinned wasmer 7.1) are recorded with the reason in `.cargo/audit.toml`.

## 2.2.0 — 2026-10-28

File manager features that Dolphin, Nautilus, Finder and Windows Explorer have
and WingDrive lacked, plus Explorer performance and folder size fixes (#286 to
#290).

### Added

- Drag files out of the Explorer into other apps on Linux: browsers, chat
  apps, editors and other file managers receive them. Dragging a selected item
  takes the whole selection. Names with spaces, `#`, `%` or accents arrive
  intact.
- Filter the open folder by name with Ctrl+Shift+F, in list, grid, media and
  column views. Text with `*`, `?` or `[` is read as a glob. Select All and
  file commands act on the filtered items only.
- Status bar under the Explorer: folder and file counts, "M of N" while a
  filter is active, the selection's count and size, and free space on the
  folder's drive.
- Ctrl+L edits the location, with the path selected. `~` and `~/…` expand to
  the home folder, and names without a leading `/` are read from the folder
  shown, with `.` and `..`.
- Middle-click or Ctrl+click on a volume, device, location or network mount in
  the sidebar, and middle-click on a folder, opens it in a new background tab.
- Drop files on a tab to copy or move them into the folder that tab shows.
  Holding them over a tab highlights it and switches to it after 400 ms (was
  one second, with no feedback).

### Fixed

- Folder sizes follow changes: creating, resizing, moving or deleting a file
  updates the size, file count and item count of every folder above it, which
  used to wait for the next full rescan.
- The list view no longer re-renders the Explorer in an endless loop, which
  made folder navigation take seconds and kept a CPU core busy even with the
  window hidden (DESK-024).
- Folder changes no longer re-render every item of the listing.
- The list view's Size column shows a folder's content size, as the grid does.
- Dragging a file toward the window edge no longer scrolls wide list views
  sideways.
- A drag cancelled with Escape no longer leaves its preview on screen.
- Folders with `%` in their name open, and their tabs show their name.

### Changed

- Dependency updates: Rust crates within their ranges (rustls 0.23.46,
  parking_lot 0.13, jsonschema 0.60 and 96 others), Bun packages,
  typescript-eslint 8.72, GitHub Actions, and Expo SDK 58 patch releases on
  mobile. Unused graph packages (sigma, graphology) were removed.

## 2.1.1 — 2026-10-23

Explorer listing fixes for the desktop file manager (#272, #275 and later).

### Fixed

- Folders outside a location list every entry. `/home` listed empty after
  `/` was opened again, `Downloads` showed only the folder on a deeper path
  that had been opened, and large folders such as `/usr/bin` and `/usr/lib`
  came back empty on the first visit.
- An empty folder is no longer scanned again on every refresh.
- The list view fills the window: the name column takes the width the other
  columns leave, columns resize without a maximum, a double-click on a
  resize handle restores the default width, and widths persist per tab.
- The created date shows when the file was created. Folders outside a
  location showed the time they were browsed, and indexed entries the time
  they were indexed. Entries indexed before this release keep the old value
  until the file changes or its location is removed and added again.
- A burst of file changes refreshes an open folder once instead of once per
  change. Every event aborted the listing in flight and requested it again,
  so copying many files into a large folder kept it from finishing.
- Folders outside a location no longer show their own inode size (such as
  "72 B") in the size column; it stays empty until folder sizes are
  calculated.
- Ephemeral jobs (browsing folders outside a location) no longer log a
  database error when they start, and job start-up messages are debug-level.

### Changed

- Dependency updates for TanStack Query, i18next and react-i18next.

## 2.1.0 — 2026-10-20

Privacy and security fixes, location cleanup, bounded logs, Expo SDK 58 on mobile and dependency updates (#259 to #273). 2.0.2 was prepared but never tagged; its changes ship here.

### Security

- The data directory is private (0700) and the device key file is readable
  only by its owner. Both were readable by every local user, which let them
  read the device key and decrypt the stored cloud credentials.
- On Linux the daemon rejects RPC connections from other local users. The
  loopback socket had no authentication, so any account on the machine could
  browse, copy or delete the user's files.
- A malformed RPC request is no longer written to the log at error level; it
  can carry credentials.
- The desktop app's local media server no longer serves files outside the
  library's sidecars: a `..` in a thumbnail URL passed its path check. Sidecar
  URLs now also need the session token, like the file endpoint.

### Fixed

- Removing a large location deletes all its entries. The subtree was deleted
  in one statement above SQLite's parameter limit, the error was ignored, and
  the entries stayed in the library.
- The watcher no longer stores files whose folder is not indexed, and a file
  deleted right after it was created no longer logs an error.
- Logs stay bounded: daemon logs keep seven days, job logs older than seven
  days are deleted, and per-file messages are debug-level.
- `wing start` and `wing restart` start the daemon through its systemd unit
  when one is installed, so systemd keeps restarting it.
- A tab whose navigation stops reaching the screen recovers by itself instead
  of needing another tab to be opened.
- The Android release logs at info level.

### Changed

- The mobile core uses jni 0.22; native callbacks no longer leak a Java
  string per event or log line.
- TypeScript 7 for ts-client and the build scripts; rusqlite 0.39 and
  jsonschema 0.59 in the developer tools.
- Mobile: Expo SDK 58 with React Native 0.88 and React 19.3, built
  against Android SDK 37.
- Desktop: zbus 6 for the single-instance hand-off and
  org.freedesktop.FileManager1; zip 9 for archives.
- tsup 9 and happy-dom 21 in the build and test tooling.

## 2.0.1 — 2026-10-19

Desktop start-up at login, location removal, and the Android APK (#255, #256).

### Fixed

- The desktop app connects at login again. The daemon opened its socket only
  after watching every location, which took minutes on large source trees;
  watches now register in the background and skip build and dependency
  directories. The app waits for a daemon that is still starting instead of
  giving up.
- Files inside folders of a tree moved into a location are watched.
- The `wing` CLI ships with the desktop app, so the commands the
  disconnected-daemon screen suggests work.
- Removing a location while it indexes no longer fails with "database is
  locked", and the location's indexing and thumbnail jobs stop with it.
- Changes inside folders the indexing rules exclude (`target`,
  `node_modules`, …) no longer get indexed when a file inside them changes.

## 2.0.0 — 2026-10-18

First stable release of WingDrive (#238 to #251).

### Wingdrop

- Send files and folders to a paired device. The receiver sees who sends
  what, accepts or declines, and accepted files land in its Wingdrop folder
  (Downloads by default) under free names, so nothing is overwritten. Either
  side can cancel mid-transfer. File menu "Send with Wingdrop", the Wingdrop
  window, an offer prompt, Settings > Services, and
  `wing network wingdrop send|pending|accept|decline|cancel` (#251).
- Paired devices sharing a hostname are addressed as `<name>-<id>` and no
  longer show this device's data (#251).

### Archives

- Zip, tar and 7z archives open as folders: browse, preview and open their
  files, or extract a single file or folder (#241, #243).
- Extracted folders appear in the index; zip archives keep each file's own
  date (#240, #243).

### Explorer

- Location totals and scan state update live; New Folder starts a rename;
  paste asks only when a name already exists; Recents has the top bar; tabs
  show tag, kind and page names (#243).
- Overview counts paired devices and shows files seen by volume browsing;
  File Kinds updates while indexing and counts filtered results correctly
  (#240, #242).

### Reliability

- Indexer batches take the SQLite write lock up front; batches were lost to
  `database is locked` (#240).
- Settings changed while the daemon runs survive its shutdown; jobs that
  cannot resume no longer stay paused after a restart (#251).
- Daemon errors show their message instead of raw JSON; status colors
  (errors, online dots) render again (#243, #251).

### Appearance

- All twenty themes save, including the T3 and community themes (#240).
- Missing Portuguese translations filled in (#243).

### Mobile

- Android app launches, pairs with the desktop and syncs libraries, with a
  working Network and Settings tab (#238, #239).

## 2.0.0-alpha.18 — 2026-10-16

### Explorer

- A move that copies (across volumes, or with the streaming strategy on one
  volume) now removes its source. Before, a streaming move on one volume left
  the file in both places and reported success (#234).
- Open With, Always Open With, Share and Reveal appear only for files on
  this device, so a paired device's path never reaches the local opener
  (#231).
- Dragging into a folder with Ctrl copies and with Shift moves; without a
  modifier the same-volume rule still decides (#231).

### Search

- Refinement pills filter by kind, extension, size, date and hidden files,
  with counts over the whole result instead of the loaded page. Hidden
  (dot-prefixed) files stay out unless the pill is on (#232).

### Devices and storage

- ZFS capacity is detected when the daemon runs without `/usr/sbin` on its
  `PATH`, as a systemd user service on Debian and Ubuntu does (#230).
- The overview's device panel lists paired devices (#231).

### Sync

- A device that deleted its copy of a library answers the other members'
  sync requests with an error instead of leaving them to time out, and logs
  one warning per connection instead of an error per request (#235).

### Performance

- Thumbnail generation runs at most four decodes at once, fewer when free
  memory is low, instead of a whole batch of 50 (#229).
- Icons and images ship as WebP, and `wing-server` no longer embeds about
  18 MB of web source maps (#236).

### Documentation

- The README lists what works today and on which platform, and drops
  at-rest encryption and safety screening, which are not implemented (#236).

## 2.0.0-alpha.17 — 2026-10-16

### Explorer

- Copy To… and Move To… in the context menu pick a destination folder in the
  system folder picker, then run the usual copy or move with conflict handling
  and undo (#226).
- Ctrl+Shift+I inverts the selection and Ctrl+S selects items by name pattern
  (`*.jpg; *.png`). Both are also in the empty-space context menu, next to a new
  Select All entry (#227).

### Inspector

- A Checksum section calculates a file's SHA-256 on request, copies it, and
  checks it against a pasted hash or `sha256sum` line (#227).

### Translations

- Crowdin sync no longer replaces Portuguese strings with English (#224).

### Build and release

- A release tag on a commit without a CI run now starts that run instead of
  failing the release (#223).
- Merging an up-to-date pull request no longer reruns CI on main, and CodeQL
  analyzes Rust only when Rust code changes (#225, #226).

## 2.0.0-alpha.16 — 2026-10-15

### Explorer

- Folders show their real content size instead of the filesystem metadata
  length. An optional "Calculate Folder Sizes" view setting measures folders
  that are not indexed yet (#205).
- Hard-linked files count once in folder sizes and file counts (#205).
- A folder opens in a new window from its context menu or with Ctrl+N (Cmd+N
  on macOS) (#217).
- Renaming or moving a folder in a folder that is not indexed now updates the
  view, keeps the folder's contents, and reindexing it no longer fails with
  "Ephemeral index not initialized" (#208).

### Desktop

- The tray menu follows the interface language, also on a hidden autostart
  launch (#206).
- About page and Wingbot links open in the system browser (#206).
- The remaining hard-coded interface strings are translated (#212).

### Devices and storage

- An unplugged drive is marked offline instead of staying online (#210).

### Reliability

- Shutting down inside the library watcher's debounce no longer reopens the
  library and leaves its lock file behind (#206).
- Folders and files indexed together with their parent are no longer missing
  from descendant queries; existing libraries are repaired by a migration
  (#207).

### Mobile

- The mobile app is available in English and Brazilian Portuguese (#213).
- Deleting a library works, a new library is selected after creation, pinned
  locations appear in Browse, and Tailscale connections are labelled (#214).

## 2.0.0-alpha.15 — 2026-10-15

### Interface (#201)

- The interface is available in Brazilian Portuguese. Pick the language in
  Settings → Appearance; every window follows it and dates use its format.
  Translations are managed in Crowdin.
- 12 new themes: the T3 Code palettes (T3 Code, T3 Chat, Grove, Ocean, Ember,
  Iris) and Catppuccin, Gruvbox, Solarized, Rosé Pine, Dracula and Tokyo Night.
  Most follow the system light/dark setting.
- The Light theme is light again: it stayed dark, and menu hover and faint text
  were unreadable.
- File, folder, device and volume icons use their light versions under light
  themes.
- Settings are redesigned: grouped sections, switches that save immediately, a
  layout that adapts to narrow windows, a theme picker with live previews,
  device pairing options, copy and open for the data directory, and device slug
  validation. The About page follows the theme.

### Desktop (#201)

- On Linux, right-click menus follow the app theme and language and open at the
  cursor. Hyprland and other Wayland compositors centered the native popups.
- The macOS-style menu bar (WingDrive, File, Edit, View) is gone on Linux, as
  on Windows. Keyboard shortcuts are unchanged.

### Reliability (#202)

- Creating a library no longer fails with `no such column: device_id`:
  migrations run in one transaction.
- `library.json` is written atomically, so concurrent writes no longer corrupt
  it and settings changed while statistics run are kept.
- A library closed right after creation is no longer reopened by the directory
  watcher, which caused `AlreadyInUse`.
- The daemon raises its open file limit at startup, avoiding `EMFILE` with many
  watched folders.
- The daemon no longer leaves a 1 MB file named `:memory:` in its working
  directory.

### Performance (#202)

- Opening a folder that is not indexed no longer rebuilds the file type
  registry for every file, which took about 25 s for 10,000 files in debug
  builds.
- The Jobs panel lists jobs without loading each job's saved state.

## 2.0.0-alpha.14 — 2026-10-14

### Performance (#199)

- Folder navigation in the Linux desktop app is about 8x faster: the median
  folder change drops from about 450 ms to about 55 ms, with about 4x less
  WebKit CPU. On Linux the main window no longer applies backdrop-filter blurs,
  because WebKitGTK re-blurred 14 controls on every folder change. The TopBar,
  tab bar and library switcher keep their opaque surfaces. macOS, Windows and
  the web app are unchanged.
- File views build the context menu once, when an item is right-clicked. Each
  visible item had mounted its own menu: a dozen mutations plus an "Open With"
  lookup over IPC per item on every folder change.

### Desktop (#198)

- Sidebar keyboard focus uses a subtle inset outline that remains fully visible,
  and its footer controls now have equal spacing.
- Middle-clicking a folder tab closes it, matching browser tab behavior.

## 2.0.0-alpha.13 — 2026-10-13

### Performance (#193)

- The desktop app no longer idles at full CPU. `jobs.list` returned silent
  browse indexers, so a client that listed one mid-run kept it at
  "Indexing 60%" forever and its spinner kept WebKitGTK's compositor busy.
  Idle webview CPU drops from 60–100% to 1–3%.
- An empty folder is polled at most 8 times instead of every 750 ms forever.
- Tabs saved by older builds with the `unknown-device` slug list their folder
  again instead of failing with "Location root path is not local".
- `daemon.log` is written again: config v7 moves the log filter off the
  pre-rename crate names.

### Previews (#193, #195)

- Folders that are not indexed show real previews for images, videos (first
  representative frame) and PDFs (first page) instead of a generic icon.
  Previews are cached by path, modification time and size. Videos and PDFs use
  the host's `ffmpeg` and `pdftoppm`; the AUR package lists `ffmpeg` and
  `poppler` as optional dependencies.

### File management (#194)

- Trash view: list, restore and permanently delete items in the system trash
  (Linux and Windows), from the sidebar footer. Moving files to the trash can
  be undone.
- Batch rename: find/replace plus a `{name}`/`{n}` pattern with a live
  preview; empty, slashed or colliding names are blocked; one undo reverts
  the whole batch.
- Find duplicates in a folder. Every set is confirmed by reading the files in
  full, symlinks are skipped, and extra copies move to the trash with undo.
- Compare two folders and merge their differences. Changed files keep the
  newer copy; the older one goes to the trash.

### CI (#195)

- Desktop-only pull requests no longer fail the daemon spawn test: the empty
  `--no-daemon` sidecar placeholder had replaced the debug `wing-daemon`.

### Dependencies (#180, #187, #188, #190, #192)

- Dependabot updates, including `zod` 4.6.5, `style-to-js` 2.0.2 and
  `eslint-plugin-jsdoc` 65; Expo SDK 57 pins held for mobile (#191).

## 2.0.0-alpha.12 — 2026-10-12

### Brand (#172)

- New WingDrive mark: the three stacked wing feathers without the drive
  spindle. Every icon is regenerated from `.github/logo.svg`: desktop (Linux,
  macOS, Windows), iOS and Android launchers, the Android splash, the docs
  favicon and the `wing` CLI banner.

### Mobile (#148, #173)

- Native dependencies are back on the versions Expo SDK 57 pins, so the iOS and
  Android builds resolve again.
- NativeWind 5 on Tailwind CSS 4, the same Tailwind major as desktop and web.
- Mobile bundles build again: `@babel/plugin-transform-runtime` 8 needed Babel 8,
  while Expo SDK 57 runs on Babel 7.

### Security (#148, #170, #175)

- CodeQL runs in advanced setup with the `security-extended` suite on actions,
  JavaScript/TypeScript, Python and Rust. Swift returns under SEC-009.
- TruffleHog scans every pull request and push for leaked secrets, and the full
  history weekly.
- `main` and the `v*` release tags are protected by repository rulesets.
- Fixed CodeQL findings: the macOS and browser adapters copied their databases
  to a predictable temporary name; an Explorer URL parameter named `__proto__`
  could reach the object prototype; macOS framework signing passed the signing
  identity through a shell.

### Extensions (#174)

- The Photos and test extensions build for WASM again, and CI now builds both.
- The SDK enables `uuid`'s browser randomness on `wasm32`, so every extension
  can create v4 UUIDs.

### Server (#176)

- The Docker image moves to Ubuntu 26.04 with FFmpeg 8.

### Toolchain and dependencies (#168, #171, #177, #178)

- Rust 1.99.0, with every crate on its latest compatible release.
- 13 Dependabot updates, including `zip` 8, `ed25519-dalek` 3, `@headlessui/react` 2,
  ESLint 10 and `react-markdown` 10; JavaScript packages updated within their
  ranges (Expo 57.0.28, Radix UI, Tauri plugins, Vite 8.3).
- `cargo xtask setup` renders its config template in-process, which drops the
  `mustache` crate and the `log` 0.3 it pulled in.

### CI (#147, #149)

- Rust gates run only for the apps a pull request touches; release jobs are
  split; typecheck uses `bun check`.
- Pull requests get path, branch, size and type labels.

## 2.0.0-alpha.11 — 2026-10-10

### Build (DEV-004, #145)

- Local Rust builds write far less to the SSD. `just dev-server`, `just cli`
  and `just build` use the same `ffmpeg,heif` features as `just dev-daemon` and
  Tauri dev, so they share one `wing-core` build instead of a 2.4 GB
  incremental cache per feature set.
- `cargo xtask setup` caps parallel build jobs at half the CPUs outside CI.
- `just clean-stale` removes incremental caches and artifacts untouched for
  7 days.
- No changes to the app itself; this release reinstalls alpha.10's behavior.

## 2.0.0-alpha.10 — 2026-10-09

### Faster opening (DESK-010, #140, #143)

- A cold `wingdrive <folder>` paints the folder's listing in about 2.1 s, down
  from about 7 s. The daemon answers in under 1 s on a fresh install, the
  startup screen leaves as soon as it does, and the window starts on the
  requested folder instead of the Overview.
- Small folders that are not indexed yet list in the first response instead of
  appearing after a refetch; large folders still fill in progressively.
- With WingDrive running, `wingdrive <folder>` hands the folder to the open
  window over D-Bus before GTK starts (33 ms in the release smoke test).
- Test and development instances keep single-instance on with their own bus
  name, so they never hand launches to the everyday WingDrive.

### Indexing (#138)

- Ephemeral scans index the whole tree without rules for file sync
  (INDEX-011), and browsing inside a recursively indexed root reuses that index,
  including through symlinks (INDEX-012).

### Memory on Arch (DESK-011)

- Measured the AUR package on Arch's WebKit 2.54.1: 518 to 538 MiB with 15
  tabs, within the 550 MiB budget. The release smoke test prints per-process
  memory.

### Dependencies (#141)

- sqlx 0.9, bip39 3, thiserror 2, x25519-dalek 3, generic-array 1.4,
  kamadak-exif 0.6, iroh-mdns-address-lookup 0.6, dirs 7, base64 0.23,
  tower-http 0.7, gix-ignore 0.24, downcast-rs 2, and semver-compatible updates
  of the Rust and JavaScript dependency trees. Pairing words are pinned to the
  BIP39 reference vectors by a test.

### Build and release (#142)

- CI saves its Rust cache from `main`, skips the Rust job for changes that
  cannot affect it, and builds test binaries once (pull requests: about 58 min
  to about 17 min).
- The release reuses its own cache, waits for the tagged commit's CI instead
  of repeating its checks, and can republish to AUR alone (`aur_only`).
- `just ci-local` runs the CI gates locally before a push.

## 2.0.0-alpha.9 — 2026-10-07

### Linux desktop as the default file manager (DESK-000, #135)

- `xdg-open` on folders, launcher arguments and `file://` URIs open WingDrive; a
  second launch opens a tab in the running window.
- Background mode in the tray and Start at login (`--hidden`).
- `org.freedesktop.FileManager1` service, so "Show in folder" from other apps
  reveals the file in WingDrive.
- Open With lists the installed applications for the file type, with Always
  Open With to change the default.
- Open Terminal Here (Shift+F4), New File (Ctrl+Alt+N), Create Link, and Unix
  permissions and group in the Inspector.
- Compress (zip, tar.gz, tar.bz2, tar.xz, tar.zst) and extract (also 7z) as
  cancellable jobs; hostile archives cannot write outside the destination.
- Network locations and devices through gvfs (SFTP, SMB, FTP, WebDAV, phones,
  cameras), with Connect to Server.

### Packaging (TAURI-015)

- `wingdrive-bin` runs on Arch's GTK, WebKit and glib and bundles only FFmpeg,
  fixing `gdbus` symbol errors and a WebKit abort on exit (477 → 198 MiB).
- Closing the app stops the daemon it started.

### Fixes

- Folder listings update live again for local folders.
- Jobs interrupted by a restart no longer stay "running" forever.
- The Job Manager shows live progress; cancelling an archive job no longer
  spins the CPU.
- The empty-space menu opens anywhere outside an item; Rename works on a
  right-clicked item that is not selected.


### Sync

- A peer no longer loses an entry's content link when a discovery-phase state
  for that entry arrives after the content-phase state. The link is kept while
  size and modification time are unchanged.
- Changes received during a periodic incremental catch-up are applied once it
  ends. Before, they stayed buffered, and a peer could end with entries
  missing their content links. (#133)

### Indexing

- The first index of a location without a volume detects and records the
  volume, instead of failing with "Location volume_id not set". (#133)
- The root entry update retries when SQLite reports "database is locked"
  instead of failing the indexing job. (#132)

### Follow-ups

- Integration tests: fixed job shutdown and job resumption test setup; the
  ts-client daemon bridge suites skip when no bridge is configured. (#136)
- Open With shows application icons and opens the whole selection at once. (#137)

## 2.0.0-alpha.8 — 2026-10-06

### Indexing (INDEX-010, #116)

- Ephemeral browsing and volume indexing reuse the persistent UUIDs of paths a
  library already indexed, instead of minting new ids that orphan tags and
  metadata. Reconciliation runs in the background after discovery, scoped per
  library, and `ResourceChanged` carries the temporary id so clients swap it in
  place.

### Sync (#117)

- Location creation sends its root entry and location with UUID foreign keys,
  so a peer no longer hits FOREIGN KEY failures when the location arrives
  before its root.
- MIME types get one identity across devices instead of a random UUID per
  device.

### Brand (BRAND-002, #118)

- Spacebot is now Wingbot, Spacedrop is now Wingdrop (`wing network
  wingdrop`), the in-repo design system is WingUI, and Spaces are Wings in
  sidebar copy, CLI output and docs. Stored and wire names stay compatible.

### Linux runtime findings (TAURI-006, #119)

- Copy and move between `local` and this device's slug or UUID stay local
  instead of failing with "Could not find node_id for device".
- A receiver that refuses a transfer replies with the reason, and the sender
  shows it. Remote job results now reach the sender.
- Rename is F2 on Linux and Windows. Menus show the platform's shortcut glyphs.
- TopBar overflow panels (Views, Sort, View Settings) render inline in the
  menu, so they no longer close before a choice registers.
- `network.spacedrop.send` fails with an explicit error instead of reporting
  a send that never happened. The native View menu with the Wingdrop demo is
  limited to debug builds.

## 2.0.0-alpha.7 — 2026-10-06

### License

- WingDrive is now licensed under **Apache-2.0**, replacing FSL-1.1-ALv2.
  Upstream Spacedrive relicensed to Apache-2.0 in `bcc124765`, whose only
  parent is the WingDrive fork point `6dfeccf21`. See [CREDITS.md](CREDITS.md)
  for the provenance and the exceptions: file-kind icons stay GPL-3.0-only and
  the SpaceUI-derived packages stay MIT. Releases up to `v2.0.0-alpha.6` stay
  under FSL-1.1-ALv2. (#114)

### Data safety on Linux (TAURI-013, #112)

- Device, volume and location cards no longer reach file commands. Before
  this, pressing Delete with a location card selected sent the location root
  to `files.delete`.
- The selection is rebuilt from the displayed items, so deleted, renamed or
  hidden entries drop out of it.
- New Folder, Paste and external drops target the active column's folder.
  They are disabled where no single folder is shown.
- Copy and move report partial outcomes: failed and skipped counts, plus the
  error messages. Skipped items no longer count as copied, and one bad source
  no longer discards the items that already finished.
- Settings that nothing read were removed. System volume auto-tracking now
  honors its setting.
- Redundancy "shared" now lists only content present on every selected
  volume.

### Runtime robustness and search (TAURI-014, WATCH-005, #113)

- Each window gets its own event subscriptions. Closing or reloading one
  window no longer cancels the streams of the others. Streams reopen with
  backoff after the daemon restarts.
- Settings, Jobs, Inspector, Quick Preview and Spacebot windows share one
  shell: toasts, dialogs, jobs, theme, live file events, an error boundary
  and daemon reconnect. The Jobs window no longer throws outside
  `JobsProvider`.
- The startup screen gives up after 45 s and shows the start and install
  controls.
- Restart Daemon reaps a crashed child daemon. On Linux the library lock
  treats zombie PIDs as gone, so the library reopens.
- Search can be scoped to the folder or the location. The scope stays put
  while typing, and input is debounced.
  - Kind filters work.
  - Results follow the sort direction and the hidden-files setting, and load
    200 at a time.
  - Folder scope no longer matches sibling folders that share a prefix, and
    no longer treats `_` and `%` as wildcards.
- Watcher: a reused ext4 inode is no longer mistaken for a rename. Renames
  update the extension, and dotted directory names keep their full name.
- Sources gets Retry buttons, and path bar folders accept dropped files.

### Fixes ported from upstream Spacedrive (FORK-005, #115)

- Transfers are verified with a blake3 hash of the full file in both
  directions. Pull transfers used to always fail their final check. A
  shorter file replacing a longer one is now truncated correctly.
- Sync pauses for 5 minutes after 3 consecutive backfill failures instead of
  retrying every few seconds.
- Paired devices reconnect after an in-process restart, and devices that
  never connected this session are redialed. Presence no longer flaps
  online/offline. Direct paths over Tailscale show as a separate connection
  method.
- Jobs no longer linger in the client after they finish. Jobs that cannot
  resume after a restart are marked failed with a reason, and a duplicate
  dispatch returns the job already running.
- Discovery never indexes the daemon's own data directory.
- The library watcher no longer opens a library that is still being created.
- The macOS shallow watch now delivers events.
- Linux integration tests were updated for stale assertions and made
  independent of the environment.

### Development

- `just dev-desktop` starts Vite before the daemon build.
- UI primitives and AI components resolve from in-tree sources.
- Auxiliary windows are shown before they are focused.

## 2.0.0-alpha.6 — 2026-10-04

First WingDrive release published on GitHub and on the AUR (`wingdrive-bin`).
See the [release page](https://github.com/bernardopg/wingdrive/releases/tag/v2.0.0-alpha.6).
