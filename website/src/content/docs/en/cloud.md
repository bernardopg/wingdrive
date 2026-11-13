---
title: Cloud volumes
description: Connect Google Drive and Google Photos, and mount S3, Dropbox, OneDrive, Azure Blob and GCS as volumes.
order: 7
group: Using WingDrive
---

WingDrive treats cloud storage as native **volumes**. A connected service shows up in the sidebar and takes the same operations as any other drive: browse, copy, move, rename, move to the trash and sync.

## Supported services

| Service | How you connect |
| --- | --- |
| **Google Drive** and **Google Photos** | Sign in with Google in the browser. Several accounts per library. |
| Amazon S3 and S3-compatible storage (Cloudflare R2, MinIO, Wasabi, Backblaze B2, DigitalOcean Spaces) | Bucket, region and access keys |
| Dropbox, OneDrive | OAuth client and refresh token |
| Azure Blob Storage | Container, account name and key |
| Google Cloud Storage | Bucket and service account JSON |

Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA and other services are planned. The roadmap lists them under the integrations epic.

## Connect a Google account

<ol class="steps">
<li>Open <strong>Settings → Integrations</strong> (<kbd>Ctrl</kbd>+<kbd>,</kbd>).</li>
<li>Under <em>Google accounts</em>, keep <strong>Google Drive</strong> and <strong>Google Photos</strong> checked, or uncheck the one you do not want.</li>
<li>Click <strong>Connect a Google account</strong>. Your browser opens Google's sign-in page.</li>
<li>Allow access. Back in WingDrive, the account's Drive appears in the sidebar as <code>Google Drive (you@gmail.com)</code>.</li>
</ol>

Repeat to add more accounts. Each one keeps its own Drive and Photos access.

<figure class="shot">
<a href="/img/docs/settings-integrations.webp"><img src="/img/docs/settings-integrations.webp" width="1400" height="900" loading="lazy" alt="Settings, Integrations page with Google accounts, cloud sync and Google sign-in client"></a>
<figcaption>Settings → Integrations: Google accounts, cloud sync and the Google sign-in client.</figcaption>
</figure>

<div class="callout warn">

<p><strong>“Sign-in needs a Google OAuth client.”</strong> Builds that ship a Google client sign in right away. If you see this message, follow the four steps on the same page: create a project in Google Cloud Console, enable the Drive, Photos Library and Photos Picker APIs, add your accounts as test users and create an OAuth client of type <em>Desktop app</em>. Paste its ID and secret under <strong>Google sign-in client</strong>.</p>

</div>

### What you can do with Drive

- **Browse live.** Folders list straight from Drive. Google Docs, Sheets and Slides show as `.docx`, `.xlsx` and `.pptx` and open in the browser.
- **Edit in your own apps.** Double-click a Drive file to open a local copy. Each save uploads it back. If the file changed in Drive meanwhile, your version is kept as a conflict copy.
- **Copy, move, rename, new folder and trash** work between local folders and Drive, in both directions. Files dragged in from the desktop upload.
- **Quick Preview** downloads the file to a cache first. **Open in Browser** opens the item on drive.google.com.

### Keep a folder in sync

<ol class="steps">
<li>In <strong>Settings → Integrations</strong>, under <em>Cloud sync</em>, click <strong>Sync a folder</strong>.</li>
<li>Pick the folder on this device and the Drive folder to pair it with.</li>
<li>Choose both ways or one way, and on demand or every N minutes.</li>
</ol>

### Google Photos

Right-click photos or videos and choose **Upload to Google Photos**, optionally into a new or existing album. **Import from Google Photos** opens Google's picker and downloads what you select at full quality.

Disconnecting an account deletes its grant here and revokes WingDrive's access at Google.

### From the CLI

```bash
wing integrations connect google
wing integrations list
wing integrations cloud-sync add Documents ~/Documents gdrive://you@gmail.com/Backup --every 60
wing integrations photos upload <account-id> ~/Pictures/Trip --new-album "Trip"
```

## Other services

<ol class="steps">
<li>On the Overview, click <strong>Add Storage</strong> and choose <strong>Cloud Storage</strong>.</li>
<li>Pick the provider.</li>
<li>Fill in the bucket or container and the credentials from the table above, then confirm. The volume appears under <em>Volumes</em>.</li>
</ol>

<div class="shots">

<figure class="shot">
<a href="/img/docs/add-storage.webp"><img src="/img/docs/add-storage.webp" width="560" height="500" loading="lazy" alt="Add Storage dialog"></a>
<figcaption>Choose <strong>Cloud Storage</strong>…</figcaption>
</figure>

<figure class="shot">
<a href="/img/docs/add-storage-cloud.webp"><img src="/img/docs/add-storage-cloud.webp" width="560" height="620" loading="lazy" alt="Cloud provider picker with S3, R2, MinIO, B2, Wasabi, Spaces, Drive, Dropbox, OneDrive and more"></a>
<figcaption>…then the provider.</figcaption>
</figure>

</div>

For Google Drive, use **Settings → Integrations** as described above: it signs in through the browser instead of asking for tokens.

From the CLI, the interactive setup walks you through it:

```bash
wing cloud
```

Or add a volume directly:

```bash
wing volume add-cloud --help
wing volume list
```

## Paths you already know

Cloud paths use the same URIs as each provider's own tools:

```text
s3://my-bucket/photos/vacation.jpg
gdrive://you@gmail.com/Documents/contract.pdf
onedrive://Documents/budget.xlsx
dropbox://Apps/WingDrive/backup.zip
azblob://container/data/export.csv
gcs://bucket/logs/app.log
```

Copy a path from WingDrive and paste it into the AWS CLI, `gsutil` or similar.

## Content identity in the cloud

WingDrive identifies files by content with BLAKE3, wherever they live. For files over 100 KB it samples only 58 KB (header, four evenly spaced chunks and footer), so identifying a 10 GB file in the cloud transfers 58 KB, not 10 GB.

## Credentials

Cloud credentials and Google grants are encrypted with XChaCha20-Poly1305 using your library key and stored in the library database. Each library keeps separate credentials. Removing a cloud volume or disconnecting an account deletes its credentials.
