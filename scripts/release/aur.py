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
depends=('glibc' 'gtk3' 'webkit2gtk-4.1' 'xdotool' 'xdg-utils' 'alsa-lib' 'libpipewire' 'jack' 'libva')
provides=('wingdrive')
conflicts=('wingdrive')
options=('!strip')
_upstream_version={version}
source=("{filename}::{base}/releases/download/v${{_upstream_version}}/{filename}"
        "LICENSE::{base}/releases/download/v${{_upstream_version}}/LICENSE")
sha256sums=('{hashes[0]}' '{hashes[1]}')

prepare() {{
  chmod +x "{filename}"
  "./{filename}" --appimage-extract > /dev/null
  find squashfs-root -type d -exec chmod 755 {{}} +
}}

package() {{
  install -d "$pkgdir/opt/wingdrive" "$pkgdir/usr/bin"
  cp -a squashfs-root/. "$pkgdir/opt/wingdrive/"
  cat > "$pkgdir/usr/bin/wingdrive" <<'WRAPPER'
#!/bin/sh
export APPDIR=/opt/wingdrive
exec "$APPDIR/AppRun" "$@"
WRAPPER
  chmod 755 "$pkgdir/usr/bin/wingdrive"
  install -Dm644 squashfs-root/usr/share/applications/*.desktop "$pkgdir/usr/share/applications/wingdrive.desktop"
  sed -i 's|^Exec=.*|Exec=wingdrive %U|; s|^TryExec=.*|TryExec=wingdrive|' "$pkgdir/usr/share/applications/wingdrive.desktop"
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
