---
title: Adapters
description: Bring notes, code and other app data into your library from the adapter store, and keep it in sync.
order: 7.5
group: Using WingDrive
---

An **adapter** brings data from another app or service into your library as a **source**: the notes of an Obsidian vault, your GitHub repositories, issues and pull requests, and more over time. Items from a source show up in search next to your files, and each sync reads only what changed.

You install adapters from the **adapter store** inside WingDrive. The catalog is also on [wingdrive.org/adapters](/adapters/).

## Install an adapter

<ol class="steps">
<li>Open <strong>Sources</strong> in the sidebar and click <strong>+</strong> (<em>Add Source</em>). The store opens with a card for every published adapter.</li>
<li>Click a card to see what the adapter does and <strong>what it can access</strong>: the files and folders it reads, the hosts it contacts, the secrets it asks for and the programs it runs.</li>
<li>Click <strong>Install</strong>. Installing means you allow that access.</li>
</ol>

<figure class="shot">
<a href="/img/adapters/store-en.webp"><img src="/img/adapters/store-en.webp" width="795" height="300" loading="lazy" alt="The adapter store with the GitHub and Obsidian cards, each with an Official badge and a two-line summary"></a>
<figcaption>The store: one card per adapter, with who made it and what it does.</figcaption>
</figure>

<figure class="shot">
<a href="/img/adapters/details-en.webp"><img src="/img/adapters/details-en.webp" width="540" height="510" loading="lazy" alt="Obsidian's details: the paths it reads, no network, and a green note saying the access is enforced on this computer"></a>
<figcaption>Before installing, the details list everything the adapter can reach.</figcaption>
</figure>

The **Official** badge marks adapters the WingDrive maintainers keep. **Verified** marks authors with reviewed adapters already published. Every adapter, official or not, is reviewed before it reaches the store.

From the command line:

```bash
wing adapters browse            # download the newest index and list every adapter
wing adapters install obsidian  # shows the permissions and asks before installing
```

## Add a source

<ol class="steps">
<li>In the adapter's details, click <strong>Add Source</strong>.</li>
<li>Fill in the form. Many adapters find your data on their own: the Obsidian adapter fills in the vault you opened last and offers the others, and the GitHub adapter fills in the user the GitHub CLI is signed in with. Folder fields have a folder picker in the desktop app.</li>
<li>Click <strong>Add &amp; Sync</strong>. The first sync runs in the background; follow it in the job manager.</li>
</ol>

<figure class="shot">
<a href="/img/adapters/form-en.webp"><img src="/img/adapters/form-en.webp" width="540" height="470" loading="lazy" alt="The add source form with the vault path /home/ana/Notes filled in and the note Found on this computer"></a>
<figcaption>The vault was found on this computer and filled in.</figcaption>
</figure>

The source then appears under **Sources**. Open it to browse its items, search inside them, sync again or delete the source.

To change a source later, open it and choose **Edit source** in its menu. A token you saved stays saved unless you type a new one. WingDrive keeps tokens and passwords in its encrypted secret store, never in plain text with the other settings.

<figure class="shot">
<a href="/img/adapters/items-en.webp"><img src="/img/adapters/items-en.webp" width="795" height="485" loading="lazy" alt="The Notes source with six notes, each with its title, path and the start of its text"></a>
<figcaption>A synced Obsidian vault: each note with its path and the start of its text.</figcaption>
</figure>

## What adapters can access

Each adapter declares what it needs, and the store shows it before you install. How strictly WingDrive holds the adapter to that list depends on the system:

| System | What applies |
| --- | --- |
| **Linux** (AppImage, `.deb`, AUR, server archives) | The adapter runs **isolated** with [bubblewrap](https://github.com/containers/bubblewrap). It sees only the paths it declares and a private folder of its own, and it has no network unless it declares hosts. The store says *Enforced on this computer*. |
| **Linux Flatpak** | WingDrive's Flatpak sandbox applies, but the adapter is not limited to the paths it declares. |
| **macOS, Windows** | Adapters are not isolated yet. The list is what the registry reviewed. |

Every adapter also stops at its time limit and at 2 GB of memory. Adapters never receive WingDrive's own settings or tokens; a token you type into a source's form goes only to that adapter.

## Updates

WingDrive checks the store 30 seconds after it starts and every six hours. When an installed adapter has an update, **Sources** in the sidebar shows *1 adapter update*, the **+** button carries a badge, and the store's **Installed** tab counts it. Open the adapter and click **Update to …**.

- An update that asks for **more access** than the installed version says so and asks you again.
- If the registry **revokes** a version (for example, it was found to leak data), WingDrive stops running it at once and the store explains why. Update or remove it.

```bash
wing adapters outdated       # installed adapters with an update or a revoked version
wing adapters update --all   # asks again when an update needs more access
```

New releases of WingDrive itself appear in **Settings → General → Updates**, with one desktop notification per version. The AppImage and the Windows installers update themselves; AUR, `.deb` and other package-manager installs tell you to update through your package manager.

## Remove an adapter

Open the adapter in the store and click **Remove**, or run `wing adapters remove <id>`. Remove its sources first if you no longer need their items: a source whose adapter is gone keeps its items but cannot sync.

## Troubleshooting

| Message | What to do |
| --- | --- |
| *Adapters run isolated with bubblewrap, which is missing or blocked here* | Install the `bubblewrap` package (`sudo pacman -S bubblewrap`, `sudo apt install bubblewrap`). In Docker, run the container with `--security-opt seccomp=unconfined`, or set `WINGDRIVE_ADAPTER_SANDBOX=off` to run adapters without isolation. |
| *This adapter needs Python 3* | Install Python 3.9 or later (`python3` on Linux and macOS, `python` on Windows), then reopen the store. |
| *Not for this system* | The adapter's author did not publish it for your operating system. |
| A sync ends with an error | The source page shows the adapter's own message, such as *vault_path is not a folder* or *GitHub rate limit reached*. Fix the field or wait, then click **Sync**. |

## Publish your own adapter

Adapters are small programs, in Python or any compiled language, that read a JSON document and print JSON lines. Read the [publishing guide](https://github.com/bernardopg/wingdrive-adapters/blob/main/CONTRIBUTING.md) and the [adapter contract](https://github.com/bernardopg/wingdrive/blob/main/docs/core/adapters.mdx), then open a pull request in the registry. Wingbot checks it and a maintainer reviews it; new adapters are announced in **#adapters** on Discord.
