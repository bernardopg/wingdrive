# TrueNAS SCALE Deployment Guide

Quick reference for deploying WingDrive Server on TrueNAS SCALE using the GUI.

## Quick Start

### 1. Build and Transfer (on your Mac)

```bash
cd ~/Projects/wingdrive/apps/server

# Build for TrueNAS
./build-for-truenas.sh

# Transfer to TrueNAS (replace with your IP)
scp wingdrive-server-*.tar.gz root@192.168.1.100:/tmp/
```

### 2. Load Image (on TrueNAS)

```bash
# SSH into TrueNAS
ssh root@192.168.1.100

# Load the image
gunzip -c /tmp/wingdrive-server-*.tar.gz | docker load

# Verify
docker images | grep wingdrive
# Should show: wingdrive-server  latest  ...
```

### 3. Deploy via GUI

Go to: **Apps** → **Discover Apps** → **Launch Docker Image**

---

## GUI Configuration

### Container Images

| Field | Value |
|-------|-------|
| Image Repository | `wingdrive-server` |
| Image Tag | `latest` |
| **Image Pull Policy** | **Never** ️ (Use local image!) |

### Container Settings

| Field | Value |
|-------|-------|
| Container Name | `wingdrive` |
| Restart Policy | Unless Stopped |

### Networking

#### Port Forwarding

| Container Port | Protocol | Node Port | Description |
|---------------|----------|-----------|-------------|
| `8080` | TCP | `8080` | Web UI & API |
| `7373` | TCP | `7373` | P2P Networking |

**Access your server at:** `http://TRUENAS-IP:8080`

### Storage

#### Primary Data Volume

| Field | Value |
|-------|-------|
| Type | Host Path (or ixVolume) |
| Host Path | `/mnt/your-pool/wingdrive` |
| Mount Path | `/data` |
| Read Only | (needs write access) |

**This stores:**
- Library databases (`.sdlibrary/`)
- Daemon socket
- Logs
- Thumbnails/sidecars

#### Optional: Media Access

Mount your existing media as read-only:

| Field | Value |
|-------|-------|
| Type | Host Path |
| Host Path | `/mnt/your-pool/media` |
| Mount Path | `/media` |
| Read Only | |

Repeat for other datasets:
- `/mnt/your-pool/photos` → `/photos`
- `/mnt/your-pool/documents` → `/documents`

### Environment Variables

**Required:**

| Name | Value | Description |
|------|-------|-------------|
| `SD_AUTH` | `admin:CHANGE_THIS_PASSWORD` | Authentication (username:password) |

**Recommended:**

| Name | Value | Description |
|------|-------|-------------|
| `TZ` | `America/New_York` | Your timezone |
| `RUST_LOG` | `info,sd_core=debug` | Log level |

**Optional:**

| Name | Value | Description |
|------|-------|-------------|
| `PORT` | `8080` | HTTP port (if you want to change it) |
| `SD_P2P` | `true` | Enable P2P (default: true) |

### Health Check (Optional)

| Field | Value |
|-------|-------|
| Type | HTTP |
| Port | `8080` |
| Path | `/health` |
| Initial Delay | `30` seconds |
| Timeout | `10` seconds |
| Period | `30` seconds |

### Resource Limits (Optional)

**Recommended for NAS stability:**

| Resource | Limit | Reservation |
|----------|-------|-------------|
| Memory | `2 GB` | `512 MB` |
| CPU | - | - |

---

## Post-Installation

### 1. Verify Container is Running

In TrueNAS GUI:
- **Apps** → **Installed** → Should see **wingdrive** with green status

Or via shell:
```bash
docker ps | grep wingdrive
```

### 2. Check Logs

In GUI: Click **wingdrive** → **Logs**

Or via shell:
```bash
docker logs wingdrive
```

Should see:
```
WingDrive Server listening on http://localhost:8080
✓ Daemon started successfully
```

### 3. Access Web UI

Open browser: `http://YOUR-TRUENAS-IP:8080`

Login with credentials from `SD_AUTH`:
- Username: `admin`
- Password: (whatever you set)

---

## Updating the Server

When you rebuild on your Mac:

### 1. Build new image

```bash
cd ~/Projects/wingdrive/apps/server
./build-for-truenas.sh
```

### 2. Transfer and load

```bash
# Transfer new tar
scp wingdrive-server-*.tar.gz root@TRUENAS-IP:/tmp/

# SSH and load
ssh root@TRUENAS-IP
gunzip -c /tmp/wingdrive-server-*.tar.gz | docker load
```

### 3. Restart container in GUI

**Apps** → **Installed** → **wingdrive** → **Stop** → **Start**

Or via shell:
```bash
docker restart wingdrive
```

The container will use the updated `wingdrive-server:latest` image.

---

## Troubleshooting

### Container won't start

**Check logs:**
```bash
docker logs wingdrive
```

**Common issues:**
- Permission denied on `/data` → Check host path exists and is writable
- Port already in use → Change `8080` to something else
- Auth error → Verify `SD_AUTH` format is `username:password`

### Can't connect to web UI

1. Verify container is running: `docker ps | grep wingdrive`
2. Check port mapping: Should show `0.0.0.0:8080->8080/tcp`
3. Test from TrueNAS shell: `curl http://localhost:8080/health`
4. Check firewall rules (TrueNAS should allow by default)

### Daemon not starting

**Check socket:**
```bash
docker exec wingdrive ls -la /data/daemon/
```

Should see `daemon.sock`

**Check daemon logs:**
```bash
docker exec wingdrive cat /data/logs/daemon.log
```

### Wrong architecture error

Make sure you built with `--platform linux/amd64`:
```bash
docker inspect wingdrive-server:latest | grep Architecture
# Should show: "Architecture": "amd64"
```

If not, rebuild:
```bash
./build-for-truenas.sh
```

---

## File Locations

**On TrueNAS host:**
```
/mnt/your-pool/wingdrive/
├── daemon/
│   └── daemon.sock          # Unix socket
├── libraries/
│   └── My Library.sdlibrary/
│       ├── library.db       # SQLite database
│       └── sidecars/        # Thumbnails
└── logs/
    └── daemon.log
```

**Inside container:**
```
/data/                       # Maps to host path above
/media/                      # Your media mounts (if configured)
/photos/
/documents/
```

---

## Advanced Configuration

### Custom Port

If port 8080 is taken:

1. Change environment variable: `PORT=9000`
2. Update port forwarding: `9000 → 9000`
3. Access at: `http://TRUENAS-IP:9000`

### Multiple Instances

Run multiple WingDrive instances with different data dirs:

**Instance 1 (Personal):**
- Container name: `wingdrive-personal`
- Host path: `/mnt/pool/wingdrive-personal`
- Ports: `8080:8080`, `7373:7373`

**Instance 2 (Work):**
- Container name: `wingdrive-work`
- Host path: `/mnt/pool/wingdrive-work`
- Ports: `8081:8080`, `7374:7373`
- Env: `INSTANCE=work`

### Reverse Proxy (HTTPS)

If you want HTTPS access, put behind nginx/Caddy:

**Caddy example:**
```
wingdrive.yourdomain.com {
    reverse_proxy localhost:8080
    basicauth {
        admin $2a$14$... # hashed password
    }
}
```

---

## Security Notes

**️ IMPORTANT:**
- **Always set SD_AUTH** - never use `SD_AUTH=disabled` on a network-accessible server
- **Use strong passwords** - not `admin:changeme`
- Consider **firewall rules** if exposing to internet
- Run behind **reverse proxy with HTTPS** for public access
- **Read-only mounts** for media you don't want WingDrive to modify

**Network access:**
- `8080` → Web UI (needs auth)
- `7373` → P2P (encrypted via QUIC/TLS)

---

## Backup

**What to backup:**
```
/mnt/your-pool/wingdrive/libraries/
```

This contains your library databases and metadata.

**How:**
- TrueNAS snapshots (recommended)
- Or periodic `rsync`/`tar` backup

**Not needed:**
- `daemon.sock` (recreated on start)
- `logs/` (optional)

---

## Support

**View logs:**
```bash
docker logs -f wingdrive
```

**Shell access:**
```bash
docker exec -it wingdrive sh
```

**Check daemon status:**
```bash
curl -u admin:yourpassword http://TRUENAS-IP:8080/health
```

Should return: `OK`

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────────┐
│ TrueNAS SCALE: WingDrive Server                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Image:  wingdrive-server:latest                           │
│ Pull:   Never (use local image)                            │
│                                                             │
│ Ports:  8080 (Web UI), 7373 (P2P)                          │
│                                                             │
│ Volume: /mnt/pool/wingdrive → /data                       │
│                                                             │
│ Env:    SD_AUTH=admin:password (REQUIRED)                  │
│         TZ=America/New_York                                 │
│                                                             │
│ Access: http://TRUENAS-IP:8080                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```
