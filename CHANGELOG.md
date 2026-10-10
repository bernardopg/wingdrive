# Changelog

All notable changes to WingDrive are recorded here. Versions follow the product
version in the workspace manifests and the matching `v*` git tag.

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
  parent is the WingDrive fork point `6dfeccf21`. See [NOTICE.md](NOTICE.md)
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
