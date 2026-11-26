---
title: Self-hosting
description: Run wing-server on a NAS, home server or VPS, with Docker or systemd.
order: 10
group: Advanced
---

`wing-server` embeds the full WingDrive daemon, with media processing, and serves the web interface and the RPC API over HTTP. Your data stays on your hardware while you reach it from any browser.

## Docker

Images exist for x86_64 and ARM64, run as a non-root user and check their own health. The compose file from the repository is the whole install:

```bash
curl -O https://raw.githubusercontent.com/bernardopg/wingdrive/main/apps/server/docker-compose.yml
WING_AUTH='me:a-long-password' docker compose up -d
```

Open `http://<server>:8080` and sign in with those credentials. Without `WING_AUTH` the file refuses to start, so there is no default password. To index existing storage, uncomment the bind mount in the file and add the folder as a location in the web UI. Update with `docker compose pull && docker compose up -d`.

The container uses host networking: other devices find the server for pairing through mDNS, and sync uses a UDP port picked at start, so publishing ports is not enough. Host networking exists on Linux only.

Without compose:

```bash
docker run -d --name wingdrive --network host --restart unless-stopped \
  -v wingdrive-data:/data -e WING_AUTH='me:a-long-password' \
  ghcr.io/bernardopg/wingdrive/server:latest
```

## Release archive

Without a container runtime, use the `wing-server-<version>-linux-x86_64` archive from the [download page](/download/) (`.tar.gz`, `.tar.xz` or `.zip`). It holds `bin/wing-server`, the `bin/wing` CLI and the FFmpeg and libheif libraries they load in `lib/wingdrive/`, and needs glibc 2.39 or newer.

```bash
VERSION=3.8.1
sudo tar -xJf wing-server-$VERSION-linux-x86_64.tar.xz -C /opt
sudo ln -sfn /opt/wing-server-$VERSION-linux-x86_64 /opt/wingdrive-server
```

Keep `bin/` and `lib/` together. To update, extract the new archive and move the `/opt/wingdrive-server` link.

## systemd

Create `/etc/systemd/system/wingdrive.service`:

```ini
[Unit]
Description=WingDrive Server
After=network.target

[Service]
Type=simple
User=wingdrive
Group=wingdrive
Environment="DATA_DIR=/var/lib/wingdrive"
Environment="WING_AUTH=admin:your-secure-password"
ExecStart=/opt/wingdrive-server/bin/wing-server --data-dir /var/lib/wingdrive
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

```bash
sudo useradd -r -s /bin/false wingdrive
sudo mkdir -p /var/lib/wingdrive
sudo chown wingdrive:wingdrive /var/lib/wingdrive
sudo systemctl enable --now wingdrive
```

## Configuration

| Setting | Meaning |
|---|---|
| `WING_AUTH` | Basic auth users, `user1:pass1,user2:pass2`. `disabled` turns auth off (not recommended on a network). |
| `DATA_DIR` | Where libraries, thumbnails and logs live. Always set it in production. |
| `PORT` / `--port` | HTTP port, 8080 by default. |

> The server refuses to start without `WING_AUTH` unless you disable authentication explicitly. This prevents accidentally exposing your libraries.

## Reverse proxy

The server binds to all interfaces. For internet access, put it behind a reverse proxy with TLS. A minimal Nginx site:

```nginx
server {
    listen 443 ssl http2;
    server_name files.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        client_max_body_size 0;
    }
}
```
