# Keep the library watcher from reopening a library closed right after creation

Branch: `bernardopg:fix/watcher-reopen-created-library` (commit `1bb1068e9`), base `spacedriveapp/spacedrive:main` at `4d41a379f`.

Follow-up to #3145.

## Problem

`LibraryManager`'s directory watcher debounces a new `.sdlibrary` folder for 500 ms. It skips the folder only while the path is in `creating`, and `CreationGuard::drop` removes the path as soon as creation finishes. The folder's own create event is still pending at that point, so if the creator closes the library inside the debounce window, the watcher opens it again. The reopened copy holds `.sdlibrary.lock`, and the next open of that library fails with `AlreadyInUse`.

Now that library migrations run in one transaction (#3147), creation is fast enough for this window to be hit in normal runs. #3145 fixed the variant where shutdown lands inside the debounce; this is the explicit `close_library` variant.

## Fix

- `creating` becomes `HashMap<PathBuf, Option<Instant>>`: `None` while the creator is building the library, `Some(t)` once the guard drops at instant `t`.
- The watcher keeps the event time of each pending create and skips it when creation is in progress or finished at or after that event. A later event for the same folder (e.g. a library copied in over it) is still processed.

## Test

`core/tests/library_test.rs::closing_a_new_library_inside_the_watcher_debounce_keeps_it_closed`: creates a library, closes it at once, waits past the debounce, and asserts it is still closed.

## How verified

- Without the fix the new test fails on every run; with it, `cargo test -p sd-core --test library_test` passes 7/7, including `stopping_the_watcher_inside_its_debounce_reopens_nothing` from #3145.
- `cargo fmt` clean; `cargo clippy -p sd-core --lib` shows no new warnings.
