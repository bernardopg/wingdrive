# What's New in 2.0.0-alpha16

Released October 15, 2026. [Full changelog](./CHANGELOG.md).

## Explorer 📁

- Folders show real content size; optional "Calculate Folder Sizes" for non-indexed folders
- Hard-linked files count once in folder sizes and file counts
- Open a folder in a new window (Ctrl+N / Cmd+N)
- Fixed rename/move in non-indexed folders — view updates and reindexes without errors

## Desktop 🖥️

- Tray menu follows interface language, including hidden autostart launches
- About page and Wingbot links open in the system browser
- Remaining hard-coded strings translated

## Devices & Storage 💾

- Unplugged drives are marked offline instead of staying online

## Reliability 🛡️

- Shutdown inside the library watcher's debounce no longer reopens the library or leaves lock files behind
- Migration repairs folders and files missing from descendant queries

## Mobile 📱

- Available in English and Brazilian Portuguese
- Delete a library, select a new library after creation, pinned locations in Browse, Tailscale labels
