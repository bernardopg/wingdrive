#!/usr/bin/env python3
"""Generate AUR metadata from the verified release artifacts.

Usage: aur.py VERSION APPIMAGE LICENSE DESTINATION [PACKAGE]

Two AUR packages carry the same AppImage. `wingdrive-bin` follows stable
releases only. `wingdrive-nightly` follows every release, alpha, beta and rc
included, so it is never older than `wingdrive-bin`. PACKAGE defaults to
`wingdrive-nightly` for a pre-release and `wingdrive-bin` otherwise; a
pre-release is refused for `wingdrive-bin`.
"""
import hashlib
import re
import sys
from pathlib import Path

PACKAGES = {
    "wingdrive-bin": "WingDrive desktop file manager",
    "wingdrive-nightly": "WingDrive desktop file manager, every release including alpha, beta and rc",
}

if len(sys.argv) not in (5, 6):
    raise SystemExit(__doc__)
version, image, license_file, destination = sys.argv[1:5]
if not re.fullmatch(r"\d+\.\d+\.\d+(?:-(?:alpha|beta|rc)\.\d+)?", version):
    raise SystemExit("Invalid release version")
prerelease = "-" in version
pkgname = sys.argv[5] if len(sys.argv) == 6 else ("wingdrive-nightly" if prerelease else "wingdrive-bin")
if pkgname not in PACKAGES:
    raise SystemExit(f"Unknown AUR package: {pkgname}")
if prerelease and pkgname == "wingdrive-bin":
    raise SystemExit("wingdrive-bin takes stable releases only; publish pre-releases as wingdrive-nightly")
# Both packages install /opt/wingdrive and /usr/bin/wing, so each conflicts with the other.
other = next(name for name in PACKAGES if name != pkgname)
filename = f"WingDrive-{version}-x86_64.AppImage"
base = "https://github.com/bernardopg/wingdrive"
hashes = [hashlib.file_digest(open(path, "rb"), "sha256").hexdigest() for path in (image, license_file)]
out = Path(destination)
out.mkdir(parents=True, exist_ok=True)
pkgbuild = f'''# Maintainer: Bernardo Gomes <bernardopg@users.noreply.github.com>
pkgname={pkgname}
pkgver={version.replace('-', '').replace('alpha.', 'alpha').replace('beta.', 'beta').replace('rc.', 'rc')}
pkgrel=1
pkgdesc='{PACKAGES[pkgname]}'
arch=('x86_64')
url='https://wingdrive.org'
license=('Apache-2.0' 'GPL-3.0-only')
depends=('glibc' 'libgcc' 'glib2' 'gtk3' 'webkit2gtk-4.1' 'libsoup3' 'cairo' 'gdk-pixbuf2' 'dbus' 'libheif' 'libva' 'xdg-utils' 'libayatana-appindicator')
optdepends=('gst-plugins-good: audio and video playback in previews'
            'gvfs: network locations (SFTP, FTP, WebDAV) in the sidebar'
            'gvfs-mtp: Android phones over MTP'
            'gvfs-smb: Windows and Samba shares'
            'gvfs-gphoto2: cameras'
            'ffmpeg: video thumbnails in folders that are not indexed'
            'poppler: PDF thumbnails in folders that are not indexed')
provides=('wingdrive')
conflicts=('wingdrive' '{other}')
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
  install -Dm755 squashfs-root/usr/bin/wing "$app/usr/bin/wing"
  install -d "$app/usr/lib"
  cp -a squashfs-root/usr/lib/WingDrive "$app/usr/lib/"
  (cd squashfs-root/usr/lib && cp -a $_bundled_libs "$app/usr/lib/")

  local missing
  missing="$(ldd "$app/usr/bin/WingDrive" "$app/usr/bin/wing-daemon" "$app/usr/bin/wing" "$app"/usr/lib/*.so.* | grep 'not found' | sort -u || true)"
  if [[ -n "$missing" ]]; then
    printf 'Unresolved libraries:\n%s\n' "$missing" >&2
    return 1
  fi

  install -d "$pkgdir/usr/bin"
  ln -s /opt/wingdrive/usr/bin/WingDrive "$pkgdir/usr/bin/wingdrive"
  # The CLI finds wing-daemon next to its resolved path, so a symlink works.
  ln -s /opt/wingdrive/usr/bin/wing "$pkgdir/usr/bin/wing"
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
X-GNOME-UsesNotifications=true
DESKTOP
  # No D-Bus activation file for org.freedesktop.FileManager1: the name
  # belongs to the user's default file manager. WingDrive claims it at runtime
  # only when it is the default for inode/directory (filemanager1.rs).
  if [[ -d squashfs-root/usr/share/icons ]]; then
    install -d "$pkgdir/usr/share/icons"
    cp -a squashfs-root/usr/share/icons/. "$pkgdir/usr/share/icons/"
  fi
  install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
  install -Dm644 squashfs-root/usr/lib/WingDrive/NOTICE "$pkgdir/usr/share/licenses/$pkgname/NOTICE"
  install -Dm644 squashfs-root/usr/lib/WingDrive/CREDITS.md "$pkgdir/usr/share/licenses/$pkgname/CREDITS.md"
  cp -a squashfs-root/usr/lib/WingDrive/licenses/. "$pkgdir/usr/share/licenses/$pkgname/"
}}
'''
(out / "PKGBUILD").write_text(pkgbuild)
print(out / "PKGBUILD")
