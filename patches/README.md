# Local crate patches

## glib-0.18.5

Linux Tauri still pulls gtk-rs GTK3 (glib 0.18.x) through wry/webkit2gtk.
Upstream fixed GHSA-wrw7-89jp-8q8g (unsound `VariantStrIter::impl_get`) only
in glib >= 0.20, so this is glib 0.18.5 with the one-line backport
(`&p` -> `&mut p` in `src/variant_iter.rs`). Remove once Tauri's Linux stack
moves to glib >= 0.20.
