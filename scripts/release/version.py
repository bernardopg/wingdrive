#!/usr/bin/env python3
"""Reject releases whose product versions disagree with their tag."""
import json
import re
import sys
import tomllib
from pathlib import Path

tag = sys.argv[1]
if not re.fullmatch(r"v\d+\.\d+\.\d+(?:-(?:alpha|beta|rc)\.\d+)?", tag):
    raise SystemExit(f"Unsupported release tag: {tag}")
version = tag[1:]
for name in ("core", "apps/server", "apps/cli", "apps/tauri/src-tauri", "apps/tauri/wing-tauri-core"):
    found = tomllib.loads(Path(name, "Cargo.toml").read_text())["package"]["version"]
    assert found == version, f"{name}: {found} != {version}"
for name in ("apps/tauri/package.json", "apps/tauri/src-tauri/tauri.conf.json"):
    found = json.loads(Path(name).read_text())["version"]
    assert found == version, f"{name}: {found} != {version}"
print(version)
