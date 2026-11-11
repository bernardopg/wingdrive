---
title: Cloud volumes
description: Mount S3, Google Drive, Dropbox, OneDrive, Azure Blob, GCS and more as volumes.
order: 7
group: Using WingDrive
---

WingDrive treats cloud storage as native **volumes**. Connect a service and it behaves like any other drive: indexed, searchable and browsable, with the same file operations.

## Supported services

Cloud support is built on [OpenDAL](https://opendal.apache.org/), which covers more than 40 services:

- **Object storage**: Amazon S3, Azure Blob, Google Cloud Storage, MinIO, Backblaze B2, Cloudflare R2
- **Consumer cloud**: Google Drive, Dropbox, OneDrive and others
- **Self-hosted and enterprise**: WebDAV, Nextcloud, Seafile and others

## Connect a service

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
gdrive://My Drive/Documents/contract.pdf
onedrive://Documents/budget.xlsx
dropbox://Apps/WingDrive/backup.zip
azblob://container/data/export.csv
gcs://bucket/logs/app.log
```

Copy a path from WingDrive and paste it into the AWS CLI, `gsutil` or similar.

## Offline browsing

Once a cloud volume is indexed, you can search it, browse thumbnails and read metadata offline. Only reading or writing file contents needs a connection.

## Content identity in the cloud

WingDrive identifies files by content with BLAKE3, wherever they live. For files over 100 KB it samples only 58 KB (header, four evenly spaced chunks and footer), so identifying a 10 GB file in the cloud transfers 58 KB, not 10 GB. A photo on Google Drive and its copy on your laptop share the same identity.

## Credentials

Cloud credentials are encrypted with XChaCha20-Poly1305 using your library key and stored in your operating system's keyring (Secret Service on Linux, Credential Manager on Windows, Keychain on macOS). Each library keeps separate credentials. Removing a cloud volume purges its credentials.
