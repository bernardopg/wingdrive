# Upstream port survey: mobile, Tauri and shared interface (2026-10-16)

Range `6dfeccf21..upstream/main` (`4d41a379f`), paths `apps/mobile apps/tauri packages/interface packages/ts-client`: **92** non-merge commits (the task estimate was 66).
Core halves are already classified in `docs/design/UPSTREAM_PORT_SURVEY.md` and `.tasks/core/FORK-005-port-upstream-fixes.md`. Rows marked "(survey)" reuse those verdicts.

Method: `git show --stat` and diffs for every commit. `git cherry-pick --no-commit` was tested in a throwaway worktree (now removed) on 15 candidates. **None applied cleanly.** Every one conflicted, mostly because of the `@sd/*` to `@wingdrive/*` and `sd_path` to `wing_path` renames, plus WingDrive's i18n.
Tokensave was unusable: the index showed 0 nodes with a rebuild in progress, then the server disconnected. WingDrive code was checked with grep and sed.

## 1. Summary

- **Mobile has not advanced upstream.** Only 6 commits touch `apps/mobile`, and all are fallout from core refactors. Net change is 15 files, +94/-365, mostly deletions: locations/spaces removed (`318ab3891`, `8a878a3c9`), device facts moved (`4e58ddca5`), and the `DaemonRequest.device` field added to `sd-mobile-core` (`960febb2b`, `61dea6055`). Mobile has nothing to port.
- **Tauri shell: little new desktop work.** The changes are:
  - Spacebot and voice overlay removed (`fec139fbd`). WingDrive keeps Spacebot.
  - Photos app window and hot-thumb HTTP routes in `server.rs` (`33a86dacd`, `90e2f3ba3`, `f0347a317`, `f3a1ede2f`, `25fd868a2`).
  - Extension viewer bundles (`26b67ff0d`).
  - A macOS asset-path alias fix (`4e53d58db`).
  - A CSP `media-src` for remote streaming (`e752cb460`).
  - A LaunchAgent fd limit (`fc1798b05`).
  - Build/CI tolerance for a missing `dist` or sidecar (`61dea6055`).
  - Regenerated capability schemas.

  WingDrive already has the fd limit, `media-src`, the cfg-gated file-opening crates and the CI `dist` workaround.
- **Shared interface: almost all of it is rebuilt on upstream's new core.** Source stores replace locations: the Add to Library modal, Sources and Storage pages, VolumeBar, and the PathBar on `paths.context`. File operations sit on the journal/preflight framework: delete, merge and dedupe dialogs, plan preview, undo, rename, organize and archive. The sidebar was cut down to one library scope (`2097fc141`). Tags moved to the stores. Thumbnails come from the hot tier. None of this applies to WingDrive's locations/entries model.
- **What still applies is a small set of UI pieces:**
  - The search refinement bar (`81accc3ad`, `3f4b67100`, `7b6f8ac9f`).
  - A local-only guard on Open With and Reveal (from `e752cb460`).
  - A window modifier-key tracker for drag-and-drop (from `313a1da12`).
  - `show_paired` in the overview DevicePanel (`9d948817e`).
  - The inset layout option (`57d969ce3`, survey).
  - One core fix the survey missed: absolute `zfs`/`zpool` paths (from `5f49a0141`).
- **Releases:** upstream publishes only a rolling `nightly` prerelease of the CLI and daemon (`sd-*` binaries for linux-x86_64, macos-aarch64 and windows-x86_64) at `4d41a379f612`. It has no desktop bundle and no mobile build. The last tagged app release is still `v2.0.0-alpha.2` (2026-02-07). The release notes list no mobile or Tauri changes.

## 2. Classification

Class: PORT / ALREADY-HAVE / N/A (NOT-APPLICABLE). Effort: S (<1 h), M (half day), L (day+). Cherry-pick column: "no" means tested and it conflicts; "n/t" means not tested, but adaptation is certain because of the `@sd`/`sd_path` renames.

| Upstream | Area | Summary | Class | Reason | Effort | Cherry-pick clean? |
|---|---|---|---|---|---|---|
| `fc1798b05` | tauri | LaunchAgent `NumberOfFiles` 65536 | ALREADY-HAVE | `apps/tauri/src-tauri/src/main.rs:1323`; daemon part ported as `c75f844c1` (survey) | - | no (4 conflicts, CI/docs) |
| `e4b4cb71f` | interface | Capture-time sort, `sortOptions.ts`, inspector place | N/A | Image facet (survey). WingDrive `SortMenu.tsx` already offers Date Taken in media view, plus sort direction | - | no (22) |
| `6dc6c2bef` | ts-client | Generated types only | N/A | Sources housekeeping (survey) | - | n/t |
| `731744277` | interface | DriveSourceCard offline catalog | N/A | Source stores (survey) | - | n/t |
| `a19553f6f` | interface | Locked-volume state in sidebar, VolumeBar, Sources | N/A | Needs L1 to L4 and peer facts (survey) | - | n/t |
| `306f97fda` | ts-client | Types only | N/A | Volume index (survey) | - | n/t |
| `26b67ff0d` | tauri, interface | Extension renderer registry, viewer bundles (`server.rs`, `BundleRenderer.tsx`) | N/A | Depends on K1/K2 `kind_name` (survey) | - | n/t |
| `0e5340394` | ts-client | Types only | N/A | Survey | - | n/t |
| `30b4f19ba` | tauri css | Tailwind `@source` for installed primitives | ALREADY-HAVE | Survey: vendored `packages/wingdrive-primitives` | - | no (2) |
| `746bebb2d` | interface | Add to Library modal replaces AddStorageModal | N/A | `sources.track` (survey) | - | n/t |
| `312dbf39b` | ts-client | Types only | N/A (UI) | Core half is PORT(idea) L in survey | - | n/t |
| `f69c1fee8` | interface | Add to Library core wiring | N/A | Survey | - | n/t |
| `b9385262f` | ts-client | Types only | N/A (UI) | Core half is PORT(idea) L in survey | - | n/t |
| `1e1fc8340` | ts-client | Types only | N/A (UI) | Core KeyManager part ported (survey) | - | n/t |
| `92dea58a4` | tauri | Regenerated `linux-schema.json`, ACL manifest | N/A (UI) | Schema churn; CI-gate idea in survey | - | n/t |
| `08eb622e2` | ts-client | Backup/restore types | N/A (UI) | Core half is PORT(idea) L in survey | - | n/t |
| `e58507a1e` | ts-client | Types only | N/A | Survey | - | n/t |
| `5bc474e72` | ts-client | Types only | N/A (UI) | Core partly ported (survey) | - | n/t |
| `10dbc18d0` | ts-client | Replica fetch types | N/A | Replica fetch | - | n/t |
| `61dea6055` | tauri, mobile, interface | CI green: `build.rs` empty `dist` and no `externalBin` for check; file-opening crates cfg-gated; mobile `device: None`; typecheck cleanup | ALREADY-HAVE / N/A | `scripts/ci-rust.sh:26` does `mkdir -p apps/tauri/dist`; WingDrive `build.rs` only panics in release; `file-opening-{macos,windows}/src/lib.rs:1` already cfg-gated; WingDrive has no `device` field; interface fixes target source code | - | no (33) |
| `9929d22c7` | tauri | Regenerated schemas | N/A | Plugin-set churn | - | n/t |
| `fec139fbd` | tauri, interface | Remove Spacebot and voice overlay | N/A | WingDrive keeps Spacebot (survey, TAURI-004) | - | n/t |
| `3cd589bc8` | tauri | Regenerated schemas | N/A | Churn | - | n/t |
| `e752cb460` | tauri, interface | Remote preview/open (`useOriginalUrl`, `files.stream_url`, `files.local_path`); Open With/Reveal/Share local-only | PORT (partial) | WingDrive `useFileContextMenu.ts:198-267` offers Open With/Reveal for any `Physical` path, including another device's, with no `is_local` check. Port the guard and the local branch of `useOriginalUrl` (dedupes 6 `convertFileSrc` uses in `ContentRenderer.tsx`). Streaming needs core ops WingDrive lacks. `media-src` already in `tauri.conf.json:49` | S (guard) / M (hook) | no (8) |
| `0c0918dcf` | ts-client | Stream types | N/A | Mounted share / peer streaming not in WingDrive (M3 idea) | - | n/t |
| `94f5e79e6` | interface | Delete preflight, batch rename, undo, mirror, organize, archive, attributes, trash view, Make Link | ALREADY-HAVE / N/A | WingDrive has its own `BatchRenameModal.tsx`, `undoStore.ts`/`useUndo.ts` plus `explorer.undo` keybind, `routes/trash`, `useArchiveActions.ts`, `PermissionsSection`. The rest is journal/preflight (survey). Make Link (symlink) is missing: PORT(idea) M with a new `files.link` op | M (idea) | n/t |
| `bcc7337b0` | interface, ts-client | Journal types, DedupeModal rename | N/A (UI) | Core is PORT(idea) L (survey) | - | n/t |
| `f410cdd4c` | interface | Duplicates screen, dedupe dialog | ALREADY-HAVE / N/A | WingDrive has `DuplicatesModal.tsx`, `routes/redundancy`; upstream code needs delete preflight | - | n/t |
| `e1e73f2b3` | ts-client | Dedupe types | N/A | Survey (`ac9f266ed` idea) | - | n/t |
| `313a1da12` | interface | Merge dialog, copy-modal preflight, plan preview, Option-drop merge, `useModifierKeys.ts` | PORT (partial) | Preflight/plan preview is N/A. `hooks/useModifierKeys.ts` is a self-contained new file. WingDrive `DndProvider.tsx:344-358` guesses move vs copy from device slug and ignores keys, so the tracker enables Ctrl=copy / Shift=move, the default in Linux file managers | S | no (13); new file copies cleanly |
| `3cd44f2da` | ts-client | Preflight client, `usePreflight` | N/A (UI) | Core is PORT(idea) L (survey) | - | n/t |
| `903c8f211` | interface | Delete compare result | N/A | Source stores; WingDrive has its own `CompareFoldersModal.tsx` | - | n/t |
| `16325fc5c` | ts-client | Types | N/A | Same | - | n/t |
| `e5e6749c2` | ts-client | Compare types | ALREADY-HAVE | `core/src/ops/files/compare.rs` + `CompareFoldersModal.tsx` | - | n/t |
| `b16816b73` | interface | Photos paging, nav focus | N/A | Source stores / Photos | - | n/t |
| `0e77655fd` | interface | Search with filters alone | ALREADY-HAVE | Survey (TAURI-014) | - | no (10) |
| `226e911d6` | interface | Tags section in sidebar | ALREADY-HAVE | `SpacesSidebar/TagsGroup.tsx` via `SpaceGroup.tsx:107`; upstream re-added what its `2097fc141` removed | - | no (9) |
| `90e2f3ba3` | tauri | Apps > Photos window (macOS Swift) | N/A | Photos app | - | n/t |
| `4e58ddca5` | mobile, interface | Device facts over peer plane | N/A | Survey | - | n/t |
| `c94eaa018` | interface | Fold sources into volume bar | N/A | Sources | - | n/t |
| `8a878a3c9` | mobile, interface | Clear location residue | N/A | Locations removal | - | n/t |
| `cc5ffc15d` | interface, ts-client | Retire "ephemeral" naming | N/A | Rename over source model | - | n/t |
| `9374855db` | tauri, ts-client | Retire persistent search/events | N/A | Locations removal | - | n/t |
| `3fd4cd315` | ts-client tests | Repoint integration tests | N/A | Sources | - | n/t |
| `318ab3891` | mobile, interface | New schema, remove locations | N/A | Survey | - | n/t |
| `ff613135e` | interface | Tag filter at stores, tag pill | N/A | Store backend. The tag pill UI folds into the search-bar port if wanted | - | n/t |
| `7d81966c3` | interface | Tags onto stores | N/A | Survey | - | n/t |
| `4cf25c17c` | ts-client | Types | N/A | Arena | - | n/t |
| `7b6f8ac9f` | interface | Scope toggle sizing | PORT | One line; only with `3f4b67100` | S | no (1) |
| `3f4b67100` | interface | Search bar from primitives: kind/size/date/ext pills, hidden switch, result count | PORT (adapt) | WingDrive `SearchToolbar.tsx` (159 lines) filters by kind only, yet `SearchFilters` (`core/src/ops/search/input.rs:56`) has `file_types`/`size_range`/`date_range`/`include_hidden`. Rewrite with `@wingdrive/primitives` and i18n | M | no (1) |
| `81accc3ad` | interface, core | Refinement panel with working filters; facets over whole match set | PORT (adapt) | UI wiring ports. Backend `SearchPage`/facets is written over upstream partitions, so re-implement in WingDrive `ephemeral_search.rs`, which applies `file_types`/`size_range`/`date_range` (lines 156-183) but not `include_hidden` | M | no (6) |
| `57d969ce3` | interface | Inset layout with floating panels option | PORT (idea) | Survey: adapt to 19 themes and i18n | M | no (6) |
| `25fd868a2` | tauri, interface | Path-scoped thumbnails, PathBar rewrite | N/A | Survey (thumbs service) | - | n/t |
| `7733fe8a9` | interface | Source scope | N/A | Sources | - | n/t |
| `f34fc3c8a` | interface | Real debounce, keep scope | ALREADY-HAVE | Survey (TAURI-014) | - | no (4) |
| `ed46af4f1` | interface | Replica sources on Storage | N/A | Replicas | - | n/t |
| `b653676e8` | interface | Polish of jobs/analyzer/redundancy/sources pages | N/A | Pages rebuilt on sources and `files.size_tree` | - | n/t |
| `c32768f2e` | interface, core | Explicit startup discovery | N/A | Volume-index snapshots; interface diff is reformatting only | - | n/t |
| `4e53d58db` | tauri, interface | Baked thumbs survive remount; macOS `/System/Volumes/Data` asset alias | PORT (partial, low) | Thumb part is hot tier (N/A). `apps/tauri/src/lib/assetPath.ts` + test is new, plus a 1-line `platform.ts` change; macOS only, not M2 | S | no (4); new files copy cleanly |
| `f3a1ede2f` | tauri, interface | Host media tool discovery, thumbstrips | N/A | Upstream thumbs service | - | n/t |
| `cf5f3f45a` | interface | `paths.context` PathBar | N/A | Locations demotion | - | n/t |
| `2097fc141` | interface | Sidebar cut down to one library scope | N/A | WingDrive keeps Spaces sidebar (design choice) | - | n/t |
| `9d948817e` | interface | `show_paired: true` in DevicePanel and sidebar | PORT | WingDrive `routes/overview/DevicePanel.tsx:107` omits it, so peers appear only after a connection event. `DevicesGroup.tsx` already passes it | S | no (1, sidebar); DevicePanel hunk applies |
| `aa589a767` | interface | Tailscale badge | ALREADY-HAVE | Ported in FORK-005 (`DevicePanel.tsx:49`) | - | n/t |
| `5f49a0141` | interface, core | Fleet stats; `zfs`/`zpool` absolute paths | PORT (core part) | UI hunk N/A (WingDrive never had `sourceFiles`). `core/src/volume/fs/zfs.rs:41,78,118,377` call bare `zfs`/`zpool`; a systemd user daemon on Debian/Ubuntu lacks `/usr/sbin` on PATH. Not in the survey | S | n/t (hand-port 30 lines) |
| `3ac1f0af7` | interface | Peer replicas in DevicePanel | N/A | Replicas | - | n/t |
| `f0347a317` | tauri | Hot-thumb 404s | N/A | Survey | - | n/t |
| `b1b0d28b7` | tauri, interface | Docs/register, schemas | N/A | Docs | - | n/t |
| `29b9b8abc` | ts-client | Remote logs types | N/A (UI) | Remote jobs already ported (`1766bd175`, FORK-005); remote logs not checked | - | n/t |
| `960febb2b` | mobile | `..` for `device` field | N/A | WingDrive has no `--device` remote ops | - | n/t |
| `7690309d8` | ts-client | Capture policy types | N/A | Sources | - | n/t |
| `f545663e1` | interface | Duplicate bar bytes | N/A | Stores | - | n/t |
| `672d472e1` | interface | Unique-bytes bar | N/A | Stores | - | n/t |
| `ac9f266ed` | ts-client | Types | N/A (UI) | Survey idea | - | n/t |
| `50ab6038e` | ts-client | Freeze source | N/A | Sources | - | n/t |
| `59520ee5e` | interface | Location to pin, delete AddLocationModal | N/A | Locations removal | - | n/t |
| `f32da4777` | interface | Tag bytes vs file | N/A | Records | - | n/t |
| `c78cf8f31` | ts-client | Delete entries types | N/A | Entries removal | - | n/t |
| `ba8efcdda` | interface | Delete media pipeline UI (FileInspector, context menu) | N/A | WingDrive keeps its media pipeline | - | n/t |
| `7d41e07ab` | ts-client | Types | N/A | Arena | - | n/t |
| `9d5465832` | interface | Stop writing entries | N/A | Entries removal | - | n/t |
| `35a8ebbbe` | ts-client | Duplicates query | N/A | WingDrive has `duplicate_detection` | - | n/t |
| `b4891e4a4` | ts-client | Types | N/A | Volume map | - | n/t |
| `c40989cb9` | interface | "Add home to library" menu | N/A | `sources.track` | - | n/t |
| `8f2d1dbf3` | ts-client | Types | N/A | Snapshots | - | n/t |
| `25be7ec66` | tauri, interface | One-click drive tracking | N/A | Sources | - | n/t |
| `0ab059813` | interface | Track drive on purpose | N/A | Sources | - | n/t |
| `831a4de09` | interface | VolumeBar 1-liner | N/A | Sources table | - | n/t |
| `60a471edc` | interface | Remove Apple Photos import | N/A | Adapter retirement | - | n/t |
| `33a86dacd` | tauri, interface | Photos app rewrite, hot thumbnail tier, `server.rs` routes | N/A | Photos/GPUI and thumbs (survey) | - | n/t |
| `35eb522e3` | interface docs | Remove hardcoded local paths | PORT (trivial) | `packages/interface/CLAUDE.md:1108` still names `/Users/jamespine/Projects/wingdrive_v1` | S | n/t (Spacebot doc conflicts; hand-edit) |
| `2ab99b036` | all | Indexed drives sidebar, settings redesign (`Settings/Layout.tsx`, `useConfig.ts`), analyzer sunburst, mobile counts | N/A | Built on ephemeral sources and `files.size_tree` (added in `7ce7862c2`, sources). WingDrive has its own settings pages and `SizeView` | - | n/t |

Totals: PORT or PORT-partial 10 (`e752cb460`, `313a1da12`, `7b6f8ac9f`, `3f4b67100`, `81accc3ad`, `57d969ce3`, `4e53d58db`, `9d948817e`, `5f49a0141`, `35eb522e3`), plus Make Link as an idea from `94f5e79e6`. ALREADY-HAVE 9. The rest are N/A.

## 3. Top 10 PORT candidates (M2 first, then mobile)

1. **Local-only Open With / Reveal / Share** (from `e752cb460`), S. Add `file.is_local` to the conditions in `packages/interface/src/routes/explorer/hooks/useFileContextMenu.ts` (Open With ~202, Reveal ~261) and to the matching inspector actions. Today a paired device's path is passed to the local opener: a wrong file opens, or nothing does.
2. **Modifier-aware drag and drop** (`useModifierKeys.ts` from `313a1da12`), S. Copy the hook as is. In `DndProvider.tsx:344-358`, Ctrl forces copy and Shift forces move; the same-volume guess remains the default. Add a test for the override logic.
3. **Search refinement bar** (`81accc3ad` + `3f4b67100` + `7b6f8ac9f`), M. Rewrite `SearchToolbar.tsx` with kind/extension/size/date pills and a hidden-files switch, using `@wingdrive/primitives` and en/pt i18n. Add `include_hidden` handling and whole-match-set facets to `core/src/ops/search/ephemeral_search.rs`. Regenerate types.
4. **`zfs`/`zpool` absolute paths** (from `5f49a0141`), S. In `core/src/volume/fs/zfs.rs`, resolve `/sbin`, `/usr/sbin` and `/usr/local/sbin` once with `OnceLock`. ZFS volume capacity then works when the daemon runs as a systemd user service.
5. **`show_paired` in overview DevicePanel** (`9d948817e`), S. One line at `packages/interface/src/routes/overview/DevicePanel.tsx:107`. Paired peers appear without waiting for a connection event.
6. **Make Link (symlink)** (idea from `94f5e79e6` `useMakeLink.ts`), M. Nautilus, Dolphin and Thunar all offer "Make Link". It needs a WingDrive `files.link` action under `core/src/ops/files/` (no preflight) plus a context-menu entry.
7. **`useOriginalUrl` hook** (local branch of `e752cb460`), M. Replace the six `convertFileSrc` copies in `QuickPreview/ContentRenderer.tsx` and `MeshViewer.tsx`. The originals load 50 ms after the thumbnail so the thumbnail draws first. Remote streaming waits for M3.
8. **Inset layout option** (`57d969ce3`, survey), M. Optional look. Touches `ShellLayout.tsx`, `SpacesSidebar/index.tsx`, `Inspector.tsx` and `AppearanceSettings.tsx`, and needs theme checks.
9. **macOS asset alias** (`4e53d58db`, `assetPath.ts` + test), S. macOS only, outside M2. Copy the files; `platform.ts` needs one line.
10. **Interface doc path cleanup** (`35eb522e3`), S. Delete the `/Users/jamespine/...` line from `packages/interface/CLAUDE.md:1108`.

**Mobile:** no upstream mobile work to port. Upstream mobile only lost the locations/spaces screens. Items 1, 3 and 5 live in `packages/interface`, but WingDrive mobile has separate screens (`apps/mobile/src/screens/...`) and gains nothing from them unless they are re-implemented there. WingDrive mobile `DevicesGroup.tsx` already passes `show_paired`.

### Suggested PRs

- **PR A, "explorer safety and DnD"** (S, about 2 h): items 1, 2 and 5. Interface only; `bun run typecheck` and bun tests.
- **PR B, "search refinement bar"** (M): item 3, core plus interface plus codegen. Track as a DESK task.
- **PR C, "Linux volume robustness"** (S): item 4. It can join the open survey item `cedc8d6f7` L1 (vanished volumes go offline) as one volume PR.
- **PR D, "Make Link"** (M): item 6, a new op and its context-menu entry.
- **PR E, "QuickPreview original URL hook"** (M): item 7, optionally with item 9.
- **Later / optional:** item 8 (inset layout); item 10 can ride along with any docs PR.

None of these cherry-pick cleanly. Re-apply each by hand and credit upstream in the commit body, as was done for `29822c72f`.
