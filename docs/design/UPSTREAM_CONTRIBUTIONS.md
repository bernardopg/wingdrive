# WingDrive fixes that can go back to upstream Spacedrive

Compared WingDrive `6dfeccf21..HEAD` (355 non-merge commits) with `upstream/main` `4d41a379f`.

**Left out:** commits that already came from upstream. These have a `cherry picked from` or `Ported/Adapted from upstream` trailer: e52b16b36, 00a797d87, a23f41083, ee86b5355, c1b9f9e7d, 82942b334, b491f1a90, e7d976bdd, 5055cdfc8, 5e2849156, 4d38d2488, 71130f1b2, e3c10e00b. Also left out: rebrand, i18n, the task tracker, CI and features that only WingDrive has.

**Duplicate check:** I searched upstream issues and PRs for every CONTRIBUTE row. No open or merged item covers the same bug. PR#2959 (closed) is a different secure-delete feature that uses encryption, not this CoW check.

## 1. Fix-by-fix table

| WingDrive sha | Fix | Upstream file(s) still affected | Class | Upstream issue/PR | Effort |
|---|---|---|---|---|---|
| 361332d93 | Refuse secure delete on CoW filesystems (btrfs/ZFS/APFS/ReFS/bcachefs) | `core/src/ops/files/delete/strategy.rs` (`secure_delete`: 3-pass overwrite, no filesystem check) | CONTRIBUTE | none (PR#2959 closed, unrelated) | M |
| 23b09f2bd | Drain the sync buffer after an incremental catch-up (success and failure) | `core/src/service/sync/mod.rs` ~L555-585 sets `Ready` directly; nothing drains `buffer` | CONTRIBUTE | none | S |
| a4ceb17ab | Directory watcher reopens a library that was just created and then closed (AlreadyInUse) | `core/src/library/manager.rs` L85-97: `CreationGuard` still removes the path from a `HashSet` on drop. #3145 fixed only the shutdown case | CONTRIBUTE | builds on PR#3145 | S |
| b55328b8c | Type extraction turns `Vec<T>` into the Swift form `[T]`, which TS reads as a 1-element tuple | `core/src/infra/wire/type_extraction.rs` L486-491; `core/src/bin/generate_typescript_types.rs`; generated `types.ts` L6076, 6095-6124 (`output: [LibraryInfo]`, etc.) | CONTRIBUTE | none | M |
| d5b619da2 (daemon/lock part) | Reap a crashed daemon; the library lock treats zombie PIDs as dead | `core/src/library/lock.rs` L133 (`ps -p` still lists zombies); `apps/tauri/src-tauri/src/main.rs` `stop_daemon_process` L915-935 calls `kill()` with no `wait()` | CONTRIBUTE | none | S |
| 7ccb3912f | Stop the daemon the app started when the app exits; keep AppImage `LD_LIBRARY_PATH` out of host tools | `apps/tauri/src-tauri/src/main.rs` L2177-2205 (`CloseRequested` only logs "Daemon will be stopped when process exits"); `apps/tauri/crates/file-opening-linux/src/lib.rs` | CONTRIBUTE | none (#2039 is unrelated, closed) | M |
| ccf7e046b (eject part) | Refuse to eject System, `/` or `/home` mounts | `core/src/ops/volumes/eject/action.rs` (no guard) | CONTRIBUTE | none | S |
| ccf7e046b (location part) | Refuse root, `/proc`, `/sys` and the data dir as a location | `ops/locations` is gone. Upstream `sources.validate_path` only gives advisory risk levels; nothing enforces them | NOT-APPLICABLE (partial) | #3017 (Windows risk flagging, related) | M |
| 949c4fbbc (transfer part) | Receiver sends back the real rejection reason instead of "transfer not found" typed as `ChecksumMismatch` | `core/src/service/network/protocol/file_transfer.rs` ~L1676 | CONTRIBUTE | none | S |
| 949c4fbbc (job_activity part) | Encode remote job events with `rmp_serde::to_vec_named` | already `to_vec_named` at `job_activity.rs` L477 | ALREADY-FIXED-UPSTREAM | — | — |
| ce940398d (reveal) | Windows `explorer /select,<path>` as a single argument; Linux reveal through `org.freedesktop.FileManager1.ShowItems` | `apps/tauri/src-tauri/src/files.rs` L131-150 (Windows passes `"/select,"` and the path as two args; Linux only `xdg-open`s the parent) | CONTRIBUTE | none | S |
| ce940398d (modal) | `FileOperationModal` calls `useLibraryQuery` inside `.slice(0,3).map`, so the hook count changes (breaks the Rules of Hooks) | `packages/interface/src/components/modals/FileOperationModal.tsx` L91-99 | CONTRIBUTE | none | S |
| 6696b48b4 | Real CodeQL findings: `tempfile.mktemp` ×6, `__proto__` injection, `exec` codesign shell injection, access-then-unlink race | `adapters/{apple-notes,chrome-history,macos-calendar,macos-contacts,opencode,safari-history}/sync.py`; `packages/interface/src/routes/explorer/context.tsx` L465; `scripts/utils/shared.mjs` L66; `packages/assets/scripts/generate.mjs` L33-39 | CONTRIBUTE | none | S |
| 3241208e0 | Backport GHSA-wrw7-89jp-8q8g to glib 0.18.5 (`[patch.crates-io]` with a vendored copy) | upstream `Cargo.lock` has glib 0.18.5 and no patch | CONTRIBUTE (maintainers may prefer another approach) | none | S |
| 551b7faea (CLI part) | `sd-cli file list` guessed directories from `size == 0` and dropped extensions | `apps/cli/src/domains/file/mod.rs` L121 | CONTRIBUTE | none | S |
| 551b7faea (order part) | Sort names case-insensitively | upstream `sort_files` already uses `to_lowercase()` | ALREADY-FIXED-UPSTREAM | — | — |
| 15372edf5 (event part) | Compare device slugs with local-alias awareness in event path filters | upstream added `SdPath::same_device`, but one literal `scope_device != res_device` is left in the `sd_path` fallback branch, `core/src/infra/event/mod.rs` ~L584 | CONTRIBUTE (residual) | none | S |
| 15372edf5 (jobs part) / 9b0d99930 | Fail or settle stale jobs and jobs that cannot resume | upstream `mark_unresumed` (`infra/job/manager.rs` L1372+) | ALREADY-FIXED-UPSTREAM | — | — |
| ac22fbab8 | Log the reason for each failed deletion | `core/src/ops/files/delete/job.rs` logs only the totals (L183) | CONTRIBUTE (low) | none | S |
| 99b8cfd1c / fd3f38b0f (closeTab part) | `setActiveTabId` called inside the `setTabs` updater (React 19 runs it during render) | `packages/interface/src/components/TabManager/TabManagerContext.tsx` L326-345 | CONTRIBUTE (low) | none | S |
| 364814abb (droppable) | Drop into a folder works only in GridView; WingDrive adds it to ListView, ColumnView and PathBar | `useDroppable` exists upstream only in `GridView/FileCard.tsx` | CONTRIBUTE (UX) | none | M |
| ce940398d (DnD op) | Drop defaults to move on the same device and copy across devices | `DndProvider.tsx` L369 hardcodes `operation: "copy"`, which upstream comments say is deliberate | CONTRIBUTE (debatable UX) | none | S |
| fee5a7f7e | Pin pairing words to the BIP39 reference vectors (test only) | `core/src/service/network/protocol/pairing/types.rs` exists | CONTRIBUTE (test) | none | S |
| 2468f0056 | Reject a copy into its own subtree | upstream `ops/files/copy/preflight.rs` L141 `CYCLE`, enforced when the action dispatches. Remaining gap: upstream does not canonicalize, so a symlink or `..` path gets past it | ALREADY-FIXED-UPSTREAM (small follow-up possible) | — | S |
| 6f68a330f | Keep the data dir out of watcher events | upstream `config::is_managed` in `handlers/fs_events.rs` L176, `phases/discovery.rs` L703, `writer.rs` L195 | ALREADY-FIXED-UPSTREAM | — | — |
| 74adc579e | Check free space before write-heavy jobs | copy: upstream `copy/preflight.rs` L197 `free_space_after`. Media-job headroom not checked | ALREADY-FIXED-UPSTREAM (copy) | — | — |
| dbda1ca06 | Linux inotify move-out and move-in; SQLite busy retry in the change handler | move halves done upstream in #3105 (`pending_move_from/to`). Busy retry is absent upstream, but its value under the source-store model is unverified | ALREADY-FIXED-UPSTREAM (moves) | PR#3105 | — |
| 1af28c82d, 5e356d657 | Count hard links once in directory aggregates | `phases/aggregation.rs` is gone. The upstream arena (`indexing/arena.rs`) sets `inode: None` and rolls up `subtree_bytes` per node, so the bug may still exist. It would need a fresh implementation | NOT-APPLICABLE (possible new issue) | none | L |
| 6255f8f0a, a22f352c6, 3c172a14d, c2e1c7a09, df12dfa46, c136f87f8, 6d7fe58b8, 01858ec37 | Closure rows, ephemeral reindex, root-entry busy retry, missing location volume, unique dir index, sync content links / mime identity, physical-path addressing | target files are gone upstream (`database_storage.rs`, `phases/processing.rs`, `ephemeral/`, `entities/entry.rs`, `File::from_entry_uuids`) | NOT-APPLICABLE | — | — |
| b0c49872f, 9070c5221, 236b47b10 | Sync test isolation and convergence waits | `core/tests/sync_backfill_race_test.rs` and `sync_realtime_test.rs` are gone upstream | NOT-APPLICABLE | — | — |
| d5b619da2 (search part) | Escape `LIKE` in folder search | upstream search no longer uses `.like(` | NOT-APPLICABLE | — | — |
| 55f2f4567, c4e0c98a5 | Delete dialog that never settles; `useWaitForJob` subscribing before dispatch | the components exist only in WingDrive (`DeleteConfirmationModal`, `useWaitForJob`) | NOT-APPLICABLE | — | — |

## 2. Recommended upstream PRs (ranked by impact for upstream users)

**Changes common to every PR:**
- Rename `WingPath`→`SdPath`, `wing_core`→`sd_core`, `.winglibrary`→`.sdlibrary`, `wing-daemon`→`sd-daemon`, `wing`→`sd-cli`, `sd_fs_watcher` (already sd), "WingDrive"→"Spacedrive".
- Upstream does not use WingDrive's i18n `t()` keys.
- Remove WingDrive task IDs (SAFE-00x, TAURI-0xx) and any `Co-Authored-By`/`Claude-Session` trailers, e.g. on d5b619da2 and 97a160ced.

1. **"Refuse secure delete on copy-on-write filesystems"**
   - From: 361332d93.
   - Change: port `delete/cow.rs` (it uses `sysinfo::Disks` and the longest mount prefix, and fails closed). Upstream also has a preflight framework: add an error `Finding` in `ops/files/delete/preflight.rs` for `DeleteMode::Secure`, or reuse `volume::types::FileSystem` (APFS/Btrfs/ZFS/ReFS are already used in `copy/routing.rs`) next to the strategy check.
   - Test: unit tests for `filesystem_is_cow` (btrfs, zfs, apfs, refs, bcachefs true; ext4, ntfs false) plus a strategy test where an unresolvable path fails closed.
2. **"Sync: apply updates buffered during incremental catch-up"**
   - From: 23b09f2bd.
   - Change: add `PeerSync::apply_buffered_updates()` and call it after both the `catchup_completed` and the `catchup_failed_but_continuing` transitions in `service/sync/mod.rs`. Check it against upstream's existing `pop_ordered` drain at `peer.rs` L2960 so the logic is not duplicated.
   - Test: a sync harness case that queues changes while in `CatchingUp` and asserts they are applied after the transition to `Ready`. In WingDrive the missing drain left 448-470 of 3170 content links missing in `test_realtime_sync_alice_to_bob`.
3. **"Library watcher must not reopen a library closed right after creation"**
   - From: a4ceb17ab. It follows up upstream #3145.
   - Change: change `creating: HashSet<PathBuf>` to `HashMap<PathBuf, Option<Instant>>`, have `CreationGuard::drop` record the instant creation finished, and have the watcher skip create events older than that instant.
   - Test: the `core/tests/library_test.rs` case from a4ceb17ab: create a library, close it at once, wait past the debounce, and assert it is still closed. It failed on every run without the fix.
4. **"Type generation: emit `T[]` for `Vec<T>` in TypeScript"**
   - From: b55328b8c.
   - Change: make type references language-aware in `infra/wire/type_extraction.rs` (keep `[T]` for Swift, emit `T[]` for TS) and use it in `bin/generate_typescript_types.rs`, then regenerate `types.ts`.
   - Test: `packages/ts-client/src/__tests__/generated-collections.test.ts`, which checks that 0, 1 and many records type-check as arrays. Add a Rust unit test for nested `Vec<Option<T>>`.
5. **"Desktop: reap and stop the daemon the app spawned; zombie PIDs do not hold the library lock"**
   - From: d5b619da2 (daemon/lock part) and 7ccb3912f (exit part).
   - Change:
     - In `library/lock.rs`, use a Linux `/proc/<pid>/stat` state check instead of `ps -p`.
     - In `apps/tauri/src-tauri/src/main.rs`, call `try_wait`/`kill`/`wait` before a restart and in `stop_daemon_process`.
     - On `RunEvent::Exit`, send SIGTERM, wait 10 s, then kill.
     - Make the AppImage env scrubbing (`file-opening-linux/src/host_env.rs`) a second commit or a separate PR.
   - Test: `process_state_is_alive` parser cases (Z/X/x, names with parentheses), plus the Linux test that spawns `sleep`, kills it and asserts it no longer counts as running before `wait()`.
6. **"Security: fix CodeQL findings (insecure temp files, prototype injection, shell injection)"**
   - From: 6696b48b4. Optionally add 3241208e0 (glib GHSA backport) as a separate PR, because it vendors a patched crate.
   - Change: the same diffs apply almost verbatim to the 6 adapters, `explorer/context.tsx`, `scripts/utils/shared.mjs` and `packages/assets/scripts/generate.mjs`. Skip `scripts/release/explorer.cjs`, which is WingDrive-only.
   - Test: an extra-params test where `?__proto__=x` does not pollute `Object.prototype`. The Python changes need no test; CodeQL re-scan is the check.
7. **"Volume eject: refuse system and root mounts"**
   - From: ccf7e046b (eject half).
   - Change: add `protected_eject_reason()` to `ops/volumes/eject/action.rs`, using `volume::types::MountType` (still upstream).
   - Test: the two unit tests from the commit (`/`, `/home` and `MountType::System` are refused; `/run/media/...` and network mounts are allowed).
8. **"Desktop small fixes: Windows/Linux reveal, FileOperationModal hook order, transfer rejection reason"**
   - From: ce940398d (reveal and modal parts), 949c4fbbc (file_transfer part), 15372edf5 residual (`same_device` at `event/mod.rs` ~L584), 551b7faea CLI part, ac22fbab8. These can be split into two or three PRs.
   - Change: the Windows reveal is a 3-line change in `files.rs`. The Linux `ShowItems` path uses `gdbus`. The modal needs fixed-length `sourceSlots`.
   - Test: an `event/mod.rs` path-scope test with a `local`-scoped subscription receiving an event that carries the real slug (from 15372edf5 `path_scope_tests`); a `file_transfer` test that a refused transfer returns `PermissionDenied` plus its reason.

## 3. Upstream open issues WingDrive already solved

WingDrive does not directly solve any of the 29 open upstream issues in a portable way. Partial or related:
- **#2916** "Sometimes ephemeral directory listings show no files". WingDrive fixed this against its own ephemeral index: 9b1d44c4c (small folders listed in the first response), 64532aef1 (empty-listing poll capped at 8), a22f352c6. Upstream rewrote listings on the volume index, so these patches do not port. The diagnosis may still help.
- **#2899** (`libavcodec.so.61` missing) and **#3069** (v2 AUR package). WingDrive ships its own AUR packaging (69aa6ff3b, ba9dc9a26, 9b7722bc4). It is fork-specific, but `scripts/release/aur.py` could serve as a template.
- **#3017** (Windows Downloads flagged as high risk). WingDrive has no fix; its location guard (ccf7e046b) uses a different list.
- Not solved by WingDrive: #3095 (FTS filters applied after pagination; WingDrive 97a160ced is frontend-only), #3086 (`df -h` locale; WingDrive `core/src/volume/platform/linux.rs` L35 is unchanged), #3056, #3008.
