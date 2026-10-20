# Local crate patches

## glib-0.18.5

Linux Tauri still pulls gtk-rs GTK3 (glib 0.18.x) through wry/webkit2gtk.
Upstream fixed GHSA-wrw7-89jp-8q8g (unsound `VariantStrIter::impl_get`) only
in glib >= 0.20, so this is glib 0.18.5 with the one-line backport
(`&p` -> `&mut p` in `src/variant_iter.rs`). Remove once Tauri's Linux stack
moves to glib >= 0.20.

## react-native-css@3.1.0-rc.0.patch

NativeWind's `react-native-css` sends every `react-native` import through
`components/index.cjs`, which lists React Native's exports one by one. React
Native 0.88 added `AssetRegistry` and the list does not have it, so
`expo-asset` read `undefined` and every font and image asset failed with
"Cannot read property 'getAssetByID' of undefined". The patch adds the getter.
Bun applies it on install (`patchedDependencies` in the root `package.json`).
Remove once a react-native-css release exports `AssetRegistry`.
