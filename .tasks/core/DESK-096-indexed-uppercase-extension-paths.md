---
id: DESK-096
title: Indexed files with an uppercase extension resolve to a path that does not exist
status: Done
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S05
tags: [core, indexing, search, bug]
last_updated: 2026-11-12
---

## Description

The indexer stores a file's extension in lowercase (`IMG_0001.JPG` becomes name `IMG_0001`, extension
`jpg`) so searches match any case. Every place that rebuilds a path from an entry joins the name and
the stored extension (`files.directory_listing`, `PathResolver`, search results), so the file comes
back as `IMG_0001.jpg`. Linux file systems are case-sensitive, so that path does not exist: opening,
copying, moving, deleting or hashing such a file from an indexed location fails. Camera photos use
`.JPG`, so whole photo folders are affected. Ephemeral (not indexed) listings keep the real path but
still show the lowercase name, and their search filter compared the extension with its case, so the
`jpg` pill hid `.JPG` files.

Each rescan also saw the file as moved (the rebuilt path is missing, the inode matches) and rewrote the
same lowercase extension.

Found on 2026-11-12 while checking DESK-020 in the web UI against an indexed location.

## Steps to reproduce

1. Isolated `wing-server`, `wing location add <folder>` for a folder with `UPPER.JPG`.
2. Open the location in the web UI. The file is listed as `UPPER.jpg`.
3. Select it and click "Calculate SHA-256" in the Inspector: `IO error at '<folder>/UPPER.jpg': No such
   file or directory (os error 2)`.

## Acceptance Criteria

- [x] New entries keep the extension's case, and listings, `PathResolver` and search give the real path
- [x] Lookups by path find rows written before the fix (lowercase extension)
- [x] A rescan rewrites old rows to the real case without losing their id
- [x] Extension filters and facets ignore case in indexed and ephemeral search
- [x] Tests cover storage, lookups, search filters and the rescan repair
- [ ] Checked in the web UI with the fixed build (the bug itself was reproduced there; see Evidence)

## Fix

- `DatabaseStorage` and `stored_name_and_extension` keep the extension as written (`JPG`), and
  `File::from_ephemeral` does the same, so every path rebuilt from name and extension is the real one.
- Lookups by path (`PathResolver::component_condition`, the persistent change detector and the
  persistent watcher handler) use `entry::extension_lookup`, which also accepts the lowercase form so
  rows indexed before the fix are still found.
- A rescan repairs old rows on its own: the rebuilt `UPPER.jpg` path is missing and the inode matches,
  so the detector reports a move and `move_entry` rewrites name and extension with their real case,
  keeping the entry id.
- Search compares extensions without case: `entry::extension_eq_ignore_case` in the SQL filters, the
  in-memory filters of persistent and ephemeral search, and the facets fold `JPG` and `jpg` into `jpg`.

## Evidence

- `test_uppercase_extensions_keep_their_real_path` in `core/tests/search_test.rs` failed before the fix
  (stored extension `jpg`) and passes after. It checks the stored extension, `PathResolver::get_full_path`,
  indexed and ephemeral search with `jpg` and `JPG` filters (2 results, one `jpg` facet, every result
  path exists), and that a rescan rewrites a lowercase row to `JPG` with the same id.
- `cargo test -p wing-core --test search_test`: 15 passed. `--test file_queries_test`: 9 passed (now
  asserts `README.MD` keeps `MD`). `--test directory_listing_first_response_test --test
  ephemeral_coverage_test`: 7 passed. `cargo test -p wing-core --lib -- stored_name_tests file_checksum
  ephemeral::index path_resolver search::`: 21 passed. `cargo clippy -p wing-core -- -D warnings` clean.
- Web UI before the fix (isolated `wing-server`, 2026-11-12): the indexed location listed `UPPER.jpg`
  and "Calculate SHA-256" failed with `No such file or directory`; in an unindexed folder the `.jpg` facet
  read 2 and the `.jpg` pill returned 1 result, dropping `photo-two.JPG`.
- Not checked in the web UI with the fixed build: the session ran out of time before rebuilding the
  server. Check it on the next package.
