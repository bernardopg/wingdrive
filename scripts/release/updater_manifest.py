#!/usr/bin/env python3
"""Write the `latest.json` the Tauri updater reads (REL-001).

Usage: updater_manifest.py <tag> <dist-dir> [repository]

Lists the signed artifacts found in <dist-dir>: the AppImage for Linux and the
NSIS installer for Windows, each with the `.sig` the build wrote next to it.
Package-manager installs (AUR, .deb) never read this file, and a platform
without a signature is left out, so an unsigned build is never offered.
"""

import datetime
import json
import pathlib
import sys

# Updater platform key -> artifact name suffix in the release assets.
PLATFORMS = {
    "linux-x86_64": "-x86_64.AppImage",
    "windows-x86_64": "-x64-setup.exe",
}


def manifest(tag: str, dist: pathlib.Path, repository: str, notes: str = "") -> dict:
    version = tag.removeprefix("v")
    platforms = {}
    for key, suffix in PLATFORMS.items():
        artifact = dist / f"WingDrive-{version}{suffix}"
        signature = artifact.with_name(artifact.name + ".sig")
        if not (artifact.is_file() and signature.is_file()):
            continue
        platforms[key] = {
            "signature": signature.read_text().strip(),
            "url": f"https://github.com/{repository}/releases/download/{tag}/{artifact.name}",
        }
    return {
        "version": version,
        "notes": notes or f"https://github.com/{repository}/releases/tag/{tag}",
        "pub_date": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }


def main() -> int:
    if len(sys.argv) not in (3, 4):
        print(__doc__, file=sys.stderr)
        return 2
    tag, dist = sys.argv[1], pathlib.Path(sys.argv[2])
    repository = sys.argv[3] if len(sys.argv) == 4 else "bernardopg/wingdrive"
    data = manifest(tag, dist, repository)
    if not data["platforms"]:
        print("No signed artifacts found; latest.json not written", file=sys.stderr)
        return 0
    (dist / "latest.json").write_text(json.dumps(data, indent=2) + "\n")
    print(f"latest.json lists {', '.join(sorted(data['platforms']))}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
