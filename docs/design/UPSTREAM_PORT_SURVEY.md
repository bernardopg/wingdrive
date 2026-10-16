# Upstream port survey, 2026-10-14

Survey of upstream Spacedrive (`spacedriveapp/spacedrive`, branch `main`)
from the WingDrive fork point `6dfeccf21` (2026-07-28) to upstream
`4d41a379f` (2026-10-13): 279 non-merge commits.

FORK-005 (`.tasks/core/FORK-005-port-upstream-fixes.md`) already classified
the first 243 commits and ported 12 of them (merged in #115). This survey
classifies the 36 commits that landed after FORK-005, re-checks the
FORK-005 "re-implement later" list, and re-checks earlier commits against
the current priority: milestone M2, WingDrive as the default file manager on
Linux (DESK-000, sprint S03).

## Classification

- **PORT**: applies cleanly or with a small adaptation to code WingDrive
  still has.
- **ALREADY-HAVE**: WingDrive already has the feature or already fixed the
  same bug.
- **NOT-APPLICABLE**: depends on upstream's source stores, arenas, volume
  index, journal/preflight framework, records/facets schema, GPUI client,
  nightly channel or Mac/NAS operations, or touches only upstream docs/CI.

Effort: S (under 1 h), M (half a day), L (one day or more).

## Ported in this branch

Branch `worktree-agent-af9207e0e2dc5b35f`, on top of `main` `a9de688ba`.

| WingDrive commit | Upstream | Area | What the user gets |
|---|---|---|---|
| `c75f844c1` | `fc1798b05` (#3153), daemon part | daemon bootstrap | The daemon raises its own open-file limit (setrlimit to the hard limit) before Core starts, instead of only warning. Watchers, SQLite pools and peer sockets no longer hit `EMFILE` under a low shell or launchd limit. launchd plists of desktop app and CLI ask for 65536. |
| `73d1cc8d6` | `ad6452dfb` (#3134) | library config | `library.json` is written atomically (stage, fsync, rename). Two concurrent writers (statistics task, config update, shutdown save) can no longer leave a corrupt file that keeps the library from opening. The statistics task no longer overwrites a config change made meanwhile. |
| `883ac9a22` | `03efcad36` (#3147) | database | Library migrations run in one transaction on one connection. A fresh library could fail to create with `no such column: device_id` when the pool handed `m20260104`'s `DROP COLUMN` to a connection with a stale schema cache; the new unit test reproduces that on WingDrive without the fix. |
| `28f8703ae` | WingDrive fix, related to `393158c23` (#3145) | library watcher | A library closed within 500 ms of its creation is no longer reopened by the library-directory watcher (which then held its lock and made the next open fail with `AlreadyInUse`). Faster migrations made `test_library_discovery` hit this race 2 to 6 times in 15 runs; with the fix 0 in 15. A new test fails every run without the fix. |
| `29822c72f` | `e053bf233`, adapted | ephemeral index | Browsing an unindexed folder no longer rebuilds the file type registry (parse of all built-in TOML) per entry: about 2.5 ms per file in debug, about 25 s for a 10,000-file folder, now about 0. |
| `ddba265e8` | `5bc474e72` (#3111), job-listing part | job manager | `jobs.list` and `get_job_info` no longer read each job's persisted `state` blob (an indexer keeps its seen-path set there). The Jobs panel stays fast with large indexer history. |
| `ac173a41d` | `1e1fc8340` (#3117), KeyManager part | crypto | `KeyManager::close` no longer creates a real 1 MB file named `:memory:` in the daemon's working directory; it uses redb's in-memory backend. |

Cherry-picks keep Jamie Pine as author, with `(cherry picked from commit …)`
and without upstream's bot co-author trailers. `29822c72f` and `28f8703ae`
are WingDrive commits that credit upstream in the body.

## Commits after FORK-005 (2026-10-05 to 2026-10-13)

| Upstream | Area | Summary | Class | Reason | Effort |
|---|---|---|---|---|---|
| `accc7235d` | docs | README: nightly downloads, beta date | NOT-APPLICABLE | Upstream product docs | - |
| `5bc474e72` | CLI, daemon, jobs, sync | Housekeeping: status defaults, `replication.paused`, sync to absent library, paired counts, resumable `sd update`, launchd 65536, `jobs.list` without state | PORT (partial) | Job listing and launchd limit ported. Replication/update/nightly parts target upstream features. Absent-library sync reply may apply to WingDrive's multiplexer: not checked | S (sync part) |
| `c81e3d616` | addressing | Content rows record their own bytes | NOT-APPLICABLE | Records/content identity schema | - |
| `179adcf6d` | tests | Source runtime acceptance matrix as CI tests | NOT-APPLICABLE | Source stores | - |
| `e58507a1e` | sources | Salvage repair, snapshot quarantine, refused watch state, nested-store search duplicates | NOT-APPLICABLE | Source stores and volume index snapshots | - |
| `08eb622e2` | libraries | Library backup and restore with blake3 manifest | PORT (idea) | Useful for a default file manager (safe upgrades), but copies source stores and `sidecars.db`; needs a rewrite over WingDrive's `library.db`/`sync.db`/sidecars | L |
| `ee76270f5` | tests | Entries-drop and file operations acceptance matrix | NOT-APPLICABLE | Tests upstream's journal/preflight file ops | - |
| `92dea58a4` | CI | Gate docs build, generated types, secrets scan, licenses | PORT (idea) | A "generated types are current" CI gate fits WingDrive's codegen rule. Workflow must be rewritten for WingDrive's CI | M |
| `1e1fc8340` | trash, store, crypto | Linux trash restore, rename over existing file, KeyManager `:memory:` | PORT (partial) | KeyManager ported. Trash fix targets upstream's journal `trash_os` (WingDrive's `ops/trash` restores `TrashItem`s from the `trash` crate directly). Store fix is source store | done |
| `cedc8d6f7` | volumes | Locked volumes L1/L2: vanished volumes go offline, attached means mounted | PORT (partial) | `VolumeMonitorService::monitor_loop` in WingDrive still only logs a tracked volume that detection no longer returns; marking it offline is portable. The volume-index half is not | M |
| `df027c29c` | volume index | One source registry per library | NOT-APPLICABLE | Volume index | - |
| `b9385262f` | extensions | Load WASM extensions in the daemon, jobs with checkpoint/resume | PORT (idea) | WingDrive keeps the same `infra/extension` code; large and touches SDK, not M2 | L |
| `5bef71f8b` | docs | Design: extension file kinds | NOT-APPLICABLE | Design doc only | - |
| `f69c1fee8` | libraries | Add to Library core | NOT-APPLICABLE | `sources.track`, store placement | - |
| `312dbf39b` | extensions | Host functions phase 1 | PORT (idea) | Follows `b9385262f` | L |
| `746bebb2d` | interface | Add to Library modal | NOT-APPLICABLE | `sources.track` | - |
| `30b4f19ba` | interface CSS | Scan the installed upstream primitives package for Tailwind classes | ALREADY-HAVE | WingDrive vendors primitives in `packages/wingdrive-primitives` and already `@source`s it | - |
| `cbdc1db3f` | CLI | `sd mcp`: MCP server over the operation registry | PORT (idea) | WingDrive has the same registry; self-contained in CLI plus `infra/wire/json_schema.rs`, but large and not M2 | L |
| `ad6452dfb` | library | Atomic `library.json` | PORT | Ported (`73d1cc8d6`) | done |
| `0e5340394` | extensions | File kinds K1+K2 | NOT-APPLICABLE | Builds on `kind_name` in upstream records | - |
| `0e7655146` | CI | Nightly release edited in place | NOT-APPLICABLE | Nightly channel | - |
| `26b67ff0d` | extensions, interface | File kinds K3 to K5: renderer registry, viewer bundles | NOT-APPLICABLE | Depends on K1/K2 | - |
| `f9d89c1d7` | arena | Arena compaction | NOT-APPLICABLE | Upstream volume-index arena | - |
| `2d781bf4b` | arena | Rebuild arena from source stores | NOT-APPLICABLE | Source stores | - |
| `306f97fda` | volumes | Locked volumes L3/L4: ZFS dataset state | NOT-APPLICABLE | Volume index events | - |
| `a19553f6f` | volumes | Locked volumes L5: state in app, CLI, peer facts | NOT-APPLICABLE | Needs L1 to L4 and peer facts | - |
| `731744277` | libraries | Offline copy of on-source catalog | NOT-APPLICABLE | Source stores | - |
| `6dc6c2bef` | housekeeping | Drop sources triggers, export stub, restore-trash prune | NOT-APPLICABLE | Source stores, backup | - |
| `b1a58be78` | extensions | Host functions phase 2 | PORT (idea) | Follows `312dbf39b` | L |
| `393158c23` | library watcher | `stop_watching` awaits the loop; stop flag checked before debounced work | PORT | WingDrive's loop still checks the flag after the tick and `stop_watching` does not await it; a shutdown inside the debounce can reopen a library `close_all` closed. Same family as `28f8703ae` | S |
| `7cab609f2` | media | EXIF into the image facet | NOT-APPLICABLE | Facets on source records | - |
| `03efcad36` | database | Migrations in one transaction | PORT | Ported (`883ac9a22`) | done |
| `3234ab28a` | tests | Two-core suites from current binary, wait on conditions | PORT (idea) | WingDrive has the same pairing suites with sleeps; rewrite is large (850 lines) | M |
| `15d7df180` | Windows | Windows build, binary naming, hidden bit, mount point, move deletes source | PORT (partial) | "Move deletes the source when it still exists after copy" may apply to WingDrive's copy job; rest is Windows/arena. Not M2 | S to M |
| `e4b4cb71f` | search | Capture time and place in listings | NOT-APPLICABLE | Image facet | - |
| `a2831ac9b` | CI | Pairing suites in CI with a local relay | PORT (idea) | WingDrive has the same suites, not in CI | M |
| `fc1798b05` | daemon | Raise fd limit | PORT | Ported (`c75f844c1`) | done |
| `4d41a379f` | macOS | `Volume.locked` build fix, macOS cargo check gate | NOT-APPLICABLE | Fixes code from L1 to L5 | - |

## FORK-005 "re-implement later" list, re-checked

| Upstream | Area | Summary | Class | Reason | Effort |
|---|---|---|---|---|---|
| `e053bf233` | indexing | File type registry built once | PORT | Ported (`29822c72f`); FORK-005 listed it under the arena, but WingDrive's ephemeral index had the same per-insert build | done |
| `4d18db0ab` | thumbnails | Bound decode workers by free memory | PORTED (FORK-006) | Re-implemented in `ops/media/thumbnail/job.rs`: the job now honours `max_concurrent` and lowers it to one generation per free GiB | M |
| `bf1ca21a1` | thumbnails | Scale video frames before selecting one | ALREADY-HAVE | WingDrive's in-process filter graph (`crates/ffmpeg/src/filter_graph.rs`) has no `thumbnail=30` frame window, and the decoder keeps libavcodec's single-thread default | - |
| `81070270d`, `63ba30edd` | indexing | statx birth time on musl | PORT (low value) | WingDrive Linux packages link glibc, where `Metadata::created()` already uses statx. Only matters for a musl build | S |
| `bcc7337b0` | file ops | Journal, trash with recorded origin, undo | PORT (idea) | Undo is a file-manager staple for M2, but upstream's code depends on its job journal; re-implement | L |
| `3cd44f2da` | file ops | Preflight for copy/move/delete, folder merge | PORT (idea) | Conflict preview before copy is valuable for M2; built on upstream's action framework; re-implement | L |
| `ac9f266ed` | dedupe | Full-byte duplicate verification | PORT (idea) | Only relevant if WingDrive adds dedupe-delete | M |
| `f9bdf724b` | sync | Library join fixes | PORT (idea) | M3 networking, kept on FORK-005 list | L |
| `5e285d3c9` | CLI | Nightly channel, sha-aware self-update | NOT-APPLICABLE | WingDrive ships through AUR/AppImage releases | - |

## Earlier commits re-checked for M2 file-manager value

| Upstream | Area | Summary | Class | Reason | Effort |
|---|---|---|---|---|---|
| `a733bca63`, `94f5e79e6` | CLI, interface | Rename, batch rename, mirror, organize, flatten, archive, attributes, trash view | NOT-APPLICABLE as code | Built on the journal/preflight framework (`bcc7337b0`); WingDrive already has rename, trash and archive ops of its own | - |
| `57d969ce3` | interface | Inset window layout with floating panels | PORT (idea) | Pure layout option in `packages/interface`; would need adapting to WingDrive's 19 themes and i18n | M |
| `0e77655fd` | search | Search with filters alone | ALREADY-HAVE | WingDrive Fast search accepts an empty query (TAURI-014) | - |
| `af3470373`, `f34fc3c8a` | search | Honest sort/pagination, real debounce | ALREADY-HAVE | TAURI-014 | - |
| `4bc442e43` | search | Case-insensitive ephemeral search | ALREADY-HAVE | Listed in FORK-005 | - |
| `b38e7deb2` | watcher | A change lands in a browsed directory that was empty | NOT-APPLICABLE | Fix is in upstream's arena fan-out; WingDrive's live listing path differs (WATCH tasks) | - |
| `0d299b247` | watcher | Make a move reach the client | NOT-APPLICABLE | Source-store fan-out; WingDrive handles moves through its own `EntryMoved` events | - |
| `f0347a317`, `4e53d58db`, `25fd868a2` | thumbnails | Hot-thumb 404s, keep baked thumbnails, path-scoped generation | NOT-APPLICABLE | Upstream thumbnail service rewrite | - |
| `226e911d6` | GPUI | Tags in sidebar | NOT-APPLICABLE | GPUI client | - |
| `ec1ff6640`, `a33fde636` | build | mimalloc and vendored openssl on musl | NOT-APPLICABLE | No musl build | - |
| `12cdfa4be` | build, assets | Shrink default build from 156 MB to 44 MB: source maps out of the asset embed, PNG icons to WebP, `wasm`/`ffmpeg`/`heif` opt-in | PORT (idea) | WingDrive still ships 196 PNG icons (including `_Light` variants); the WebP conversion and source-map exclusion are portable. Making `wasm` opt-in conflicts with WingDrive's `core` default. Diff is tangled with upstream's crate split, so redo rather than cherry-pick | M |

## Remaining commits (bulk NOT-APPLICABLE)

All other commits between `6dfeccf21` and `4d41a379f` are NOT-APPLICABLE for
the reasons FORK-005 records: source stores and arenas (`da4d8dd4d` through
`b4891e4a4`, `9718a179b`, `63828b23d`, `9d5465832` to `318ab3891` and the
follow-ups), records/content identity and tags on stores, the volume index
map, Photos app and GPUI client, thumbnail service rewrite, replica fetch and
peer replicas, location removal, Spacebot removal (`fec139fbd`; WingDrive
keeps Spacebot), docs moves and NAS/Mac runbooks. The 12 commits FORK-005
ported are listed there.

## Top remaining PORT candidates

Ranked by value for M2 against effort:

1. `393158c23`: library watcher stop awaits its loop (S). Closes the
   shutdown variant of the reopen race fixed in `28f8703ae`.
2. `cedc8d6f7` L1, monitor part: mark tracked volumes that vanished as
   offline (M). An unplugged USB drive stays "online" in the sidebar today.
3. `4d18db0ab` + `bf1ca21a1`: memory-bounded thumbnail decoding and scaled
   video frames, re-implemented in WingDrive's thumbnail job (M). Done
   2026-10-16 (FORK-006); the scaling half was already in place.
4. `15d7df180`, move part: delete a moved source when it still exists after
   the copy strategy (S to M; check whether WingDrive's copy job has the gap).
5. `5bc474e72`, sync part: answer sync traffic for an absent library once
   per connection (S).
6. `92dea58a4`: CI gate that generated TypeScript types are current (M).
7. `bcc7337b0` / `3cd44f2da`: undo and copy preflight, as WingDrive
   features rather than ports (L each).
