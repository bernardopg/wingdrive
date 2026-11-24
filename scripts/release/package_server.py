#!/usr/bin/env python3
"""Packs the server and command line binaries into the release archives.

Usage:
  package_server.py linux <version> <release-dir> <native-lib-dir> <out-dir>
  package_server.py windows <version> <release-dir> <native-dll-dir> <out-dir>

linux writes wing-server-<version>-linux-x86_64.{tar.gz,tar.xz,zip}:

  wing-server-<version>-linux-x86_64/
    bin/wing-server  bin/wing           RUNPATH $ORIGIN/../lib/wingdrive
    lib/wingdrive/*.so.N                 the native libraries they load from <native-lib-dir>
    LICENSE  NOTICE  README.md

windows writes wing-<version>-windows-x64.zip with wing.exe, wing-daemon.exe, the DLLs of
<native-dll-dir>, LICENSE, NOTICE and README.md in one folder.

Entries carry the commit time (SOURCE_DATE_EPOCH) and fixed owners, so the same build gives
the same archives. The release workflow extracts each archive in an empty environment and
starts the binaries before publishing (docs/core/releases.mdx, "Server and command line").
"""

from __future__ import annotations

import os
import pathlib
import shutil
import subprocess
import sys
import tarfile
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
EPOCH = int(os.environ.get("SOURCE_DATE_EPOCH", "0")) or 315532800  # 1980-01-01, zip's floor

README = """# WingDrive {version} — {title}

{body}

Documentation: https://wingdrive.org/docs
Source and issues: https://github.com/bernardopg/wingdrive
Checksums: SHA256SUMS on the release page.
"""

LINUX_BODY = """Start the server with its web interface on port 8080:

    ./bin/wing-server --data-dir ~/.local/share/wingdrive-server --auth admin:change-me

Open http://localhost:8080. `--auth disabled` turns authentication off for local tests only.
`./bin/wing --help` lists the command line operations; point it at the same data directory
with `--data-dir`. The binaries find their media libraries in ../lib/wingdrive, so keep the
folder layout when you move them. Requires glibc 2.39 or newer (Ubuntu 24.04, Debian 13,
Fedora 40, Arch)."""

WINDOWS_BODY = """`wing.exe` is the command line and `wing-daemon.exe` the background service it talks to.
Keep the DLLs next to them. Run `wing.exe --help` to begin; `wing.exe start` starts the daemon.
The desktop app ships both programs already; this archive is for scripts and servers."""


def run(*args: str, env: dict | None = None) -> str:
    return subprocess.run(args, check=True, capture_output=True, text=True, env=env).stdout


# Libraries the binaries open with dlopen at run time, so ldd never lists them.
# PDF thumbnails load pdfium from ../lib/wingdrive (crates/images/src/pdf.rs).
DLOPENED = ("libpdfium.so",)


def native_libraries(binaries: list[pathlib.Path], lib_dir: pathlib.Path) -> list[pathlib.Path]:
    """Every library under lib_dir that the binaries load, directly or not, by soname."""
    env = dict(os.environ, LD_LIBRARY_PATH=str(lib_dir))
    found: dict[str, pathlib.Path] = {}
    for binary in binaries:
        for line in run("ldd", str(binary), env=env).splitlines():
            parts = line.split()
            if "not found" in line:
                raise SystemExit(f"{binary.name}: {line.strip()}")
            if len(parts) >= 3 and parts[1] == "=>":
                path = pathlib.Path(parts[2])
                if path.resolve().is_relative_to(lib_dir.resolve()):
                    found[parts[0]] = path
    for name in DLOPENED:
        path = lib_dir / name
        if not path.is_file():
            raise SystemExit(f"{name} is missing from {lib_dir}")
        found[name] = path
    return [found[name] for name in sorted(found)]


def add_tree(stage: pathlib.Path, archive_root: str, writer) -> None:
    for path in sorted(stage.rglob("*")):
        writer(path, f"{archive_root}/{path.relative_to(stage).as_posix()}")


def write_archives(stage: pathlib.Path, name: str, out: pathlib.Path, formats: list[str]) -> list[pathlib.Path]:
    written = []

    def tar_filter(info: tarfile.TarInfo) -> tarfile.TarInfo:
        info.uid = info.gid = 0
        info.uname = info.gname = "root"
        info.mtime = EPOCH
        return info

    for kind in formats:
        target = out / f"{name}.{kind}"
        if kind in ("tar.gz", "tar.xz"):
            mode = "w:gz" if kind == "tar.gz" else "w:xz"
            with tarfile.open(target, mode, format=tarfile.PAX_FORMAT) as archive:
                archive.add(stage, arcname=name, recursive=False, filter=tar_filter)
                add_tree(
                    stage,
                    name,
                    lambda path, arcname: archive.add(path, arcname=arcname, recursive=False, filter=tar_filter),
                )
        elif kind == "zip":
            with zipfile.ZipFile(target, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:

                def add(path: pathlib.Path, arcname: str) -> None:
                    info = zipfile.ZipInfo(arcname + ("/" if path.is_dir() else ""), date_time=(1980, 1, 1, 0, 0, 0))
                    mode = path.stat().st_mode & 0o777
                    info.external_attr = ((0o040000 if path.is_dir() else 0o100000) | mode) << 16
                    info.create_system = 3  # Unix, so unzip restores the permissions
                    if path.is_dir():
                        archive.writestr(info, b"")
                    else:
                        info.compress_type = zipfile.ZIP_DEFLATED
                        archive.writestr(info, path.read_bytes())

                add(stage, name)
                add_tree(stage, name, add)
        else:
            raise SystemExit(f"unknown archive format {kind}")
        written.append(target)
    return written


def common_files(stage: pathlib.Path, version: str, title: str, body: str) -> None:
    shutil.copy2(ROOT / "LICENSE", stage / "LICENSE")
    shutil.copy2(ROOT / "NOTICE", stage / "NOTICE")
    (stage / "README.md").write_text(README.format(version=version, title=title, body=body), encoding="utf-8")


def linux(version: str, release: pathlib.Path, lib_dir: pathlib.Path, out: pathlib.Path) -> list[pathlib.Path]:
    name = f"wing-server-{version}-linux-x86_64"
    stage = out / name
    shutil.rmtree(stage, ignore_errors=True)
    (stage / "bin").mkdir(parents=True)
    (stage / "lib" / "wingdrive").mkdir(parents=True)
    binaries = []
    for program in ("wing-server", "wing"):
        target = stage / "bin" / program
        shutil.copy2(release / program, target)
        target.chmod(0o755)
        run("patchelf", "--set-rpath", "$ORIGIN/../lib/wingdrive", str(target))
        binaries.append(target)
    for library in native_libraries(binaries, lib_dir):
        target = stage / "lib" / "wingdrive" / library.name
        shutil.copy2(library.resolve(), target)
        target.chmod(0o644)
    common_files(stage, version, "server and command line for Linux x86_64", LINUX_BODY)
    written = write_archives(stage, name, out, ["tar.gz", "tar.xz", "zip"])
    shutil.rmtree(stage)
    return written


def windows(version: str, release: pathlib.Path, dll_dir: pathlib.Path, out: pathlib.Path) -> list[pathlib.Path]:
    name = f"wing-{version}-windows-x64"
    stage = out / name
    shutil.rmtree(stage, ignore_errors=True)
    stage.mkdir(parents=True)
    for program in ("wing.exe", "wing-daemon.exe"):
        shutil.copy2(release / program, stage / program)
    for dll in sorted(dll_dir.glob("*.dll")):
        shutil.copy2(dll, stage / dll.name)
    common_files(stage, version, "command line and daemon for Windows x64", WINDOWS_BODY)
    written = write_archives(stage, name, out, ["zip"])
    shutil.rmtree(stage)
    return written


def main(argv: list[str]) -> int:
    if len(argv) != 5 or argv[0] not in ("linux", "windows"):
        print(__doc__, file=sys.stderr)
        return 2
    kind, version, release, native, out = argv[0], argv[1], *map(pathlib.Path, argv[2:])
    out.mkdir(parents=True, exist_ok=True)
    written = (linux if kind == "linux" else windows)(version.removeprefix("v"), release, native, out)
    for path in written:
        print(f"{path.name}: {path.stat().st_size} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
