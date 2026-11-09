# Local crate patches

## sqlx-sqlite-0.9.0

SQLx 0.9.0 caps `libsqlite3-sys` below 0.38, preventing the SQLite fix for
CVE-2026-119375. This copy widens the bound to below 0.40, following upstream
SQLx commits 94aafe3a and 10462fa4. Remove it when a SQLx release includes
those changes.

## glib-0.18.5

Linux Tauri still pulls gtk-rs GTK3 (glib 0.18.x) through wry/webkit2gtk.
Upstream fixed GHSA-wrw7-89jp-8q8g (unsound `VariantStrIter::impl_get`) only
in glib >= 0.20, so this is glib 0.18.5 with the one-line backport
(`&p` -> `&mut p` in `src/variant_iter.rs`). Remove once Tauri's Linux stack
moves to glib >= 0.20.

## proc-macro-error2-2.0.1

`wasmer-derive` (Wasmer 7.1, pinned in `core/Cargo.toml`; 7.5 still uses it)
depends on `proc-macro-error2` 2.0.1, the last release, unmaintained since
2024-09. Its `__export` module re-exports the private `extern crate
proc_macro`, which rustc flags as E0365 ("will be rejected by a future version
of Rust", rust-lang/rust#127909) on every build. This is 2.0.1 with the
compiler's suggested one-line fix (`extern crate proc_macro;` ->
`pub extern crate proc_macro;` in `src/lib.rs`). Remove once a
`proc-macro-error2` release fixes it
(GnomedDev/proc-macro-error-2#13) or Wasmer drops the dependency.

## react-native-css@3.1.0-rc.1.patch

NativeWind's `react-native-css` sends every `react-native` import through
`components/index.cjs`, which lists React Native's exports one by one. React
Native 0.88 added `AssetRegistry` and the list does not have it, so
`expo-asset` read `undefined` and every font and image asset failed with
"Cannot read property 'getAssetByID' of undefined". The patch adds the getter.
Bun applies it on install (`patchedDependencies` in the root `package.json`).
Remove once a react-native-css release exports `AssetRegistry`.
