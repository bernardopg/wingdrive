# Apply changes buffered during an incremental catch-up

Branch: `bernardopg:fix/sync-apply-buffered-changes` (commit `4a742b64e`), base `spacedriveapp/spacedrive:main` at `4d41a379f`.

## Problem

The sync loop in `core/src/service/sync/mod.rs` runs a periodic incremental catch-up. It moves the device to `CatchingUp`, so every change received from peers while the catch-up runs goes into the buffer (`on_state_change_received` / `on_shared_change_received` check `should_buffer()`). When the catch-up ends, success or failure, the loop sets `Ready` directly. Only `PeerSync::transition_to_ready` drains the buffer, and this path never calls it, so those changes are never applied.

On a busy pair this shows up as a receiver that has the entries and content identities but is missing some of the rows that link them, with nothing in the logs.

## Fix

- New `PeerSync::finish_catch_up(peer, reason)`: sets `Ready`, logs the same transition line as before, releases the state lock, then applies what was buffered.
- Both catch-up outcomes (`catchup_completed`, `catchup_failed_but_continuing`) in the sync loop call it instead of setting `Ready` inline.
- The drain is bounded to the number of items present when it starts. `apply_state_change` pushes an update back into the buffer when it cannot extract the missing dependency's UUID, so an unbounded `while let Some(..) = pop_ordered()` could spin.

## Test

`core/tests/sync_catchup_test.rs::test_changes_buffered_during_catch_up_are_applied`: puts a library's `PeerSync` in `CatchingUp`, receives a shared `space` insert (buffered, not in the DB), calls `finish_catch_up`, and asserts the device is `Ready` and the space row exists.

## How verified

- `cargo test -p sd-core --test sync_catchup_test`: 4 passed (new test plus the existing `test_catch_up_with_empty_peer_runs_once`).
- With the drain disabled the new test fails at the "never applied" assertion.
- `cargo fmt` clean; `cargo clippy -p sd-core --lib` shows no new warnings.
