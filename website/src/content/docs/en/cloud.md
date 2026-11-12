---
title: Cloud volumes
description: Connect Google Drive and Google Photos, and mount S3, Dropbox, OneDrive, Azure Blob and GCS as volumes.
order: 7
group: Using WingDrive
---

WingDrive treats cloud storage as native **volumes**. A connected service shows up in the sidebar and works with the same operations as any other drive: browse, copy, move, rename, delete to the trash and sync.

## Supported services

| Service | How you connect |
| --- | --- |
| **Google Drive** and **Google Photos** | Sign in with Google in the browser. Several accounts per library. |
| Amazon S3 and S3-compatible storage (Cloudflare R2, MinIO, Wasabi, Backblaze B2, DigitalOcean Spaces) | Bucket, region and access keys |
| Dropbox, OneDrive | OAuth client and refresh token |
| Azure Blob Storage | Container, account name and key |
| Google Cloud Storage | Bucket and service account JSON |

Nextcloud, WebDAV, iCloud Drive, Box, pCloud, MEGA and other services are planned. The roadmap lists them under the integrations epic.

## Google Drive and Google Photos

Open **Settings > Integrations** and choose **Connect a Google account**. Your browser opens Google's sign-in page; after you allow access, the account's Drive appears in the sidebar as `Google Drive (your@email)`. Repeat to add more accounts.

- Drive folders list live. Google Docs, Sheets and Slides appear as `.docx`, `.xlsx` and `.pptx`, and open in the browser.
- Double-click a Drive file to open a local copy in your usual app. Each save uploads it back. If the file changed in Drive meanwhile, your version is kept as a conflict copy.
- **Cloud sync** keeps a folder on your computer in step with a Drive folder, both ways or one way, on demand or every few minutes.
- Right-click photos or videos and choose **Upload to Google Photos**. **Import from Google Photos** opens Google's picker and downloads what you select at full quality.
- Disconnecting an account revokes WingDrive's access at Google.

From the CLI:

```bash
wing integrations connect google
wing integrations list
wing integrations cloud-sync add Documents ~/Documents gdrive://you@gmail.com/Backup --every 60
wing integrations photos upload <account-id> ~/Pictures/Trip --new-album "Trip"
```

Google sign-in needs an OAuth client. Builds that include one sign in right away; otherwise create a "Desktop app" client in Google Cloud Console and paste it in **Settings > Integrations > Google sign-in client**.

## Other services

In the desktop app, use **Add Storage** in the sidebar. From the CLI, the interactive setup walks you through it:

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
