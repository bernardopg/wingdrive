#!/usr/bin/env python3
"""Generate AUR metadata from the verified release artifacts."""
import hashlib
import re
import sys
from pathlib import Path

version, image, license_file, destination = sys.argv[1:]
if not re.fullmatch(r"\d+\.\d+\.\d+(?:-(?:alpha|beta|rc)\.\d+)?", version):
    raise SystemExit("Invalid release version")
filename = f"WingDrive-{version}-x86_64.AppImage"
base = "https://github.com/bernardopg/wingdrive"
hashes = [hashlib.file_digest(open(path, "rb"), "sha256").hexdigest() for path in (image, license_file)]
out = Path(destination)
out.mkdir(parents=True, exist_ok=True)
pkgbuild = f'''# Maintainer: Bernardo Gomes <bernardopg@users.noreply.github.com>
pkgname=wingdrive-bin
pkgver={version.replace('-', '').replace('alpha.', 'alpha').replace('beta.', 'beta').replace('rc.', 'rc')}
pkgrel=1
pkgdesc='WingDrive desktop file manager'
arch=('x86_64')
url='{base}'
license=('Apache-2.0' 'GPL-3.0-only')
depends=('glibc' 'libgcc' 'glib2' 'gtk3' 'webkit2gtk-4.1' 'libsoup3' 'cairo' 'gdk-pixbuf2' 'dbus' 'libheif' 'libva' 'xdg-utils' 'libayatana-appindicator')
optdepends=('gst-plugins-good: audio and video playback in previews'
            'gvfs: network locations (SFTP, FTP, WebDAV) in the sidebar'
            'gvfs-mtp: Android phones over MTP'
            'gvfs-smb: Windows and Samba shares'
            'gvfs-gphoto2: cameras')
provides=('wingdrive')
conflicts=('wingdrive')
options=('!strip' '!debug')
_upstream_version={version}
source=("{filename}::{base}/releases/download/v${{_upstream_version}}/{filename}"
        "LICENSE::{base}/releases/download/v${{_upstream_version}}/LICENSE")
sha256sums=('{hashes[0]}' '{hashes[1]}')

# The AppImage carries an Ubuntu GTK/WebKit stack. Loading it next to Arch's
# libraries breaks host tools (gdbus) and crashes WebKit on exit, so only the
# FFmpeg build the daemon links against is kept; everything else comes from
# the system.
_bundled_libs='libavcodec.so.* libavfilter.so.* libavformat.so.* libavutil.so.* libpostproc.so.* libswresample.so.* libswscale.so.*'

prepare() {{
  chmod +x "{filename}"
  "./{filename}" --appimage-extract > /dev/null
  find squashfs-root -type d -exec chmod 755 {{}} +
}}

package() {{
  local app="$pkgdir/opt/wingdrive"
  install -Dm755 squashfs-root/usr/bin/WingDrive "$app/usr/bin/WingDrive"
  install -Dm755 squashfs-root/usr/bin/wing-daemon "$app/usr/bin/wing-daemon"
  install -d "$app/usr/lib"
  cp -a squashfs-root/usr/lib/WingDrive "$app/usr/lib/"
  (cd squashfs-root/usr/lib && cp -a $_bundled_libs "$app/usr/lib/")

  local missing
  missing="$(ldd "$app/usr/bin/WingDrive" "$app/usr/bin/wing-daemon" "$app"/usr/lib/*.so.* | grep 'not found' | sort -u || true)"
  if [[ -n "$missing" ]]; then
    printf 'Unresolved libraries:\n%s\n' "$missing" >&2
    return 1
  fi

  install -d "$pkgdir/usr/bin"
  ln -s /opt/wingdrive/usr/bin/WingDrive "$pkgdir/usr/bin/wingdrive"
  install -Dm644 /dev/stdin "$pkgdir/usr/share/applications/wingdrive.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=WingDrive
GenericName=File Manager
Comment=Browse, organize and sync your files
Exec=wingdrive %U
TryExec=wingdrive
Icon=WingDrive
Terminal=false
StartupWMClass=WingDrive
Categories=System;FileTools;FileManager;
Keywords=files;folders;file manager;explorer;browser;
MimeType=inode/directory;application/x-wingdrive-memory;
DESKTOP
  # Starts WingDrive hidden when an app reveals a file while it is not running.
  install -Dm644 /dev/stdin "$pkgdir/usr/share/dbus-1/services/com.wingdrive.FileManager1.service" <<'DBUS'
[D-BUS Service]
Name=org.freedesktop.FileManager1
Exec=/usr/bin/wingdrive --hidden
DBUS
  if [[ -d squashfs-root/usr/share/icons ]]; then
    install -d "$pkgdir/usr/share/icons"
    cp -a squashfs-root/usr/share/icons/. "$pkgdir/usr/share/icons/"
  fi
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 squashfs-root/usr/lib/WingDrive/NOTICE.md "$pkgdir/usr/share/licenses/$pkgname/NOTICE.md"
  cp -a squashfs-root/usr/lib/WingDrive/licenses/. "$pkgdir/usr/share/licenses/$pkgname/"
}}
'''
(out / "PKGBUILD").write_text(pkgbuild)
print(out / "PKGBUILD")
