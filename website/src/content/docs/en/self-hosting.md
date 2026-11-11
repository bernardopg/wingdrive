---
title: Self-hosting
description: Run wing-server on a NAS, home server or VPS, with Docker or systemd.
order: 10
group: Advanced
---

`wing-server` embeds the full WingDrive daemon, with media processing, and serves the web interface and the RPC API over HTTP. Your data stays on your hardware while you reach it from any browser.

## Docker

Images exist for x86_64 and ARM64 and run as a non-root user.

```bash
docker run -d \
  --name wingdrive \
  -p 8080:8080 \
  -p 7373:7373 \
  -v wingdrive-data:/data \
  -e WING_AUTH=admin:your-secure-password \
  ghcr.io/bernardopg/wingdrive/server:latest
```

Port 8080 serves HTTP; port 7373 is for peer-to-peer connections with your devices. Mount more volumes to index existing storage.

## Docker Compose

```yaml
services:
  wingdrive:
    image: ghcr.io/bernardopg/wingdrive/server:latest
    ports:
      - "8080:8080"
      - "7373:7373"
    volumes:
      - wingdrive-data:/data
      - /mnt/media:/media:ro
    environment:
      WING_AUTH: "admin:your-password"
      TZ: "America/Sao_Paulo"
    restart: unless-stopped

volumes:
  wingdrive-data:
```

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
ExecStart=/usr/local/bin/wingdrive-server --data-dir /var/lib/wingdrive
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
