---
title: First run
description: Create a library, add your first folders and learn the interface.
order: 3
group: Start
---

WingDrive runs a background service (the **daemon**) and a separate window. Closing the window does not stop indexing or sync. With *Keep running in the background* on, WingDrive stays in the tray, so folders open instantly.

This page takes about five minutes. At the end you have a library with your first folders indexed.

<figure class="shot">
<a href="/img/shots/WingOverview-1600.webp"><img src="/img/shots/WingOverview-1600.webp" srcset="/img/shots/WingOverview-800.webp 800w, /img/shots/WingOverview-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="The Overview page with storage totals, devices, locations and volumes"></a>
<figcaption>The Overview after the first folders are indexed: storage, indexed files, devices, locations and volumes.</figcaption>
</figure>

## 1. Create a library

<ol class="steps">
<li>Open WingDrive. On first launch the setup wizard opens.</li>
<li>Type a name for the library, for example <em>Personal</em> or <em>Work</em>.</li>
<li>Confirm. The sidebar appears with <strong>Overview</strong>, <strong>Recents</strong> and <strong>Favorites</strong>.</li>
</ol>

A library is a local database about your files: names, sizes, content identity, tags and devices. Your files are never moved into it. To back up everything you organized, copy the library folder. You can create more libraries later and switch between them from the library menu next to the page title.

## 2. Add your folders

A **location** is a folder that WingDrive watches and indexes. Adding one never moves or copies files.

<ol class="steps">
<li>On the Overview, click <strong>Add Storage</strong> at the top right. You can also click <strong>+</strong> next to <em>Locations</em> in the sidebar.</li>
<li>Choose <strong>Local Folder</strong> and pick a folder. Start with something small, like Documents or Pictures.</li>
<li>Pick an indexing mode and confirm. The folder shows up under <em>Locations</em> and indexing starts.</li>
</ol>

<figure class="shot">
<a href="/img/docs/add-storage.webp"><img src="/img/docs/add-storage.webp" width="560" height="500" loading="lazy" alt="Add Storage dialog with Local Folder, Cloud Storage and External Drive"></a>
<figcaption><strong>Add Storage</strong> adds a local folder, a cloud service or an external drive.</figcaption>
</figure>

| Indexing mode | What it reads | Use it for |
|---|---|---|
| **Shallow** | Names and basic metadata | Very large folders you only browse |
| **Content** | Adds BLAKE3 content identity | Most folders: duplicates and copies are recognized |
| **Deep** | Adds media analysis and thumbnails | Photo and video folders |

Follow progress in the job manager (the sync icon at the bottom of the sidebar). Removable drives you plug in show up under *Volumes* on their own.

<div class="callout tip">

<p><strong>Tip.</strong> Indexing rules live in <strong>Settings → Indexer</strong>: skip system files, Git folders, <code>node_modules</code>-style build folders and hidden files, or honor <code>.gitignore</code>. They apply to locations you add after changing them.</p>

</div>

<figure class="shot">
<a href="/img/docs/settings-indexer.webp"><img src="/img/docs/settings-indexer.webp" width="1400" height="640" loading="lazy" alt="Indexer settings with exclusion and filter switches"></a>
<figcaption>Settings → Indexer decides what new locations skip.</figcaption>
</figure>

## 3. Find your way around

<figure class="shot">
<a href="/img/shots/WingGridView-1600.webp"><img src="/img/shots/WingGridView-1600.webp" srcset="/img/shots/WingGridView-800.webp 800w, /img/shots/WingGridView-1600.webp 1400w" sizes="(max-width: 960px) 100vw, 760px" width="1400" height="900" loading="lazy" alt="Grid view with the Inspector open on a photo"></a>
<figcaption>The Explorer in grid view. The sidebar is on the left and the Inspector on the right.</figcaption>
</figure>

- **Sidebar**: Overview, Recents, Favorites, File Kinds, locations, volumes, devices and tags. Drag items to reorder them. The palette button at the bottom adds groups.
- **Explorer**: the file view, with tabs and grid, list, columns, media and size views.
- **Inspector** (<kbd>Ctrl</kbd>+<kbd>I</kbd>): details, dates, path, permissions, SHA-256 and tags of the selection.
- **Search** (the magnifier in the top bar): finds files across every indexed location.
- **Command palette** (<kbd>Ctrl</kbd>+<kbd>Shift</kbd>+<kbd>P</kbd>): every action by name.
- **Settings** (<kbd>Ctrl</kbd>+<kbd>,</kbd> or the gear at the bottom of the sidebar).

## 4. Make it your default

On Linux, WingDrive can open folders for `xdg-open`, answer "Show in folder" from other apps and pick files for browser uploads. See [Linux integration](/docs/linux-integration/).

## Next

- [Use the Explorer](/docs/explorer/)
- [File operations and undo](/docs/file-operations/)
- [Connect Google Drive and other clouds](/docs/cloud/)
