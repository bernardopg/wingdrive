#!/usr/bin/env python3
"""Writes the GitHub Release body for a tag and checks that every asset is present.

Usage:
  notes.py <tag> <dist-dir> <owner/repo> <output.md> [--previous <tag>] [--offline]

The body has a fixed layout, documented in docs/core/releases.mdx ("Release page"):

  1. The CHANGELOG.md section of the version, and of any version tagged after the
     previous published release that never got a release page of its own.
  2. Downloads: one table per audience (desktop, server and command line, packaging),
     built from the files in <dist-dir>. A file the tables do not know goes under
     "Other files", so nothing is hidden.
  3. Verification: SHA256SUMS and the signed updater files.
  4. Pull requests, grouped by their Conventional Commits type, from GitHub's
     generated notes, inside a collapsed block.

The script exits 1 when <dist-dir> lacks an asset that every release of this kind must
ship (see `required_assets`), so the publish job never creates an incomplete release.
`--offline` skips the GitHub API (pull requests and previous release lookup) for local
previews.
"""

from __future__ import annotations

import json
import os
import pathlib
import re
import sys
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import changelog  # noqa: E402

API = "https://api.github.com"

# (pattern, platform, package, how to install). {v} is the version. Order is display order.
DESKTOP = [
    ("WingDrive-{v}-x86_64.AppImage", "Linux x86_64", "AppImage", "Portable, updates itself"),
    ("WingDrive-{v}-amd64.deb", "Linux x86_64", "Debian, Ubuntu", "`sudo apt install ./WingDrive-{v}-amd64.deb`"),
    (
        "WingDrive-{v}-x86_64.flatpak",
        "Linux x86_64",
        "Flatpak",
        "`flatpak install --user WingDrive-{v}-x86_64.flatpak`",
    ),
    ("WingDrive-{v}-x64-setup.exe", "Windows x64", "Installer", "Updates itself"),
    ("WingDrive-{v}-x64.msi", "Windows x64", "MSI", "For managed installs"),
    ("WingDrive-{v}-aarch64.dmg", "macOS Apple silicon", "Disk image", "Not notarized; see the note below"),
    ("WingDrive-{v}-android-arm64-v8a.apk", "Android arm64", "APK", "Sideload"),
    ("WingDrive-{v}-android.aab", "Android", "App bundle", "For app stores"),
]
SERVER = [
    ("wing-server-{v}-linux-x86_64.tar.gz", "Linux x86_64", "Server and CLI", "tar.gz"),
    ("wing-server-{v}-linux-x86_64.tar.xz", "Linux x86_64", "Server and CLI", "tar.xz"),
    ("wing-server-{v}-linux-x86_64.zip", "Linux x86_64", "Server and CLI", "zip"),
    ("wing-{v}-windows-x64.zip", "Windows x64", "CLI and daemon", "zip"),
]
PACKAGING = [
    ("PKGBUILD", "Arch Linux", "AUR recipe of `wingdrive-bin`"),
    ("SRCINFO", "Arch Linux", "`.SRCINFO` of `wingdrive-bin` (GitHub drops the leading dot)"),
    ("SHA256SUMS", "All", "SHA-256 of every file on this page"),
    ("latest.json", "Desktop", "Updater manifest read by installed apps"),
    ("LICENSE", "All", "Apache License 2.0"),
]
SIGNATURES = ("WingDrive-{v}-x86_64.AppImage.sig", "WingDrive-{v}-x64-setup.exe.sig")

# Conventional Commits type of a pull request title -> heading, in display order.
PR_GROUPS = [
    ("feat", "Features"),
    ("fix", "Fixes"),
    ("perf", "Performance"),
    ("security", "Security"),
    ("refactor", "Refactoring"),
    ("docs", "Documentation"),
    ("test", "Tests"),
    ("ci", "CI and build"),
    ("build", "CI and build"),
    ("deps", "Dependencies"),
    ("release", "Release"),
    ("chore", "Maintenance"),
]
TITLE = re.compile(r"^(?P<type>[a-z]+)(?:\((?P<scope>[^)]*)\))?!?:")
PR_LINE = re.compile(r"^\* (?P<title>.+) by @(?P<author>\S+) in (?P<url>https://\S+)$")


def required_assets(version: str) -> list[str]:
    """Files every release must ship. Pre-releases have no MSI (WiX needs numeric versions)."""
    names = [
        "WingDrive-{v}-x86_64.AppImage",
        "WingDrive-{v}-amd64.deb",
        "WingDrive-{v}-x86_64.flatpak",
        "WingDrive-{v}-x64-setup.exe",
        "WingDrive-{v}-aarch64.dmg",
        "wing-server-{v}-linux-x86_64.tar.gz",
        "wing-server-{v}-linux-x86_64.tar.xz",
        "wing-server-{v}-linux-x86_64.zip",
        "wing-{v}-windows-x64.zip",
        "PKGBUILD",
        "SRCINFO",
        "SHA256SUMS",
        "LICENSE",
    ]
    if "-" not in version:
        names.append("WingDrive-{v}-x64.msi")
    # The Android build signs and uploads only when its keystore secrets exist (REL-001).
    if os.environ.get("WINGDRIVE_REQUIRE_ANDROID") == "true":
        names += ["WingDrive-{v}-android-arm64-v8a.apk", "WingDrive-{v}-android.aab"]
    return [name.format(v=version) for name in names]


def human_size(size: int) -> str:
    for unit in ("B", "KB", "MB", "GB"):
        if size < 1024 or unit == "GB":
            return f"{size:.0f} {unit}" if unit == "B" else f"{size:.1f} {unit}"
        size /= 1024
    raise AssertionError


def github(path: str, token: str, body: dict | None = None):
    request = urllib.request.Request(
        f"{API}{path}",
        data=json.dumps(body).encode() if body is not None else None,
        headers={
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {token}",
            "X-GitHub-Api-Version": "2022-11-28",
        },
        method="POST" if body is not None else "GET",
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def previous_release(repo: str, tag: str, token: str) -> str | None:
    """Newest published release older than `tag`; a stable tag skips pre-releases."""
    current = changelog.version_key(tag.removeprefix("v"))
    stable = "-" not in tag
    candidates = []
    for page in range(1, 6):
        releases = github(f"/repos/{repo}/releases?per_page=100&page={page}", token)
        if not releases:
            break
        for release in releases:
            name = release["tag_name"]
            if release["draft"] or name == tag or not re.fullmatch(rf"v{changelog.VERSION}", name):
                continue
            if stable and release["prerelease"]:
                continue
            key = changelog.version_key(name.removeprefix("v"))
            if key < current:
                candidates.append((key, name))
    return max(candidates)[1] if candidates else None


def changelog_part(version: str, previous: str | None) -> str:
    _, sections = changelog.parse(changelog.CHANGELOG.read_text(encoding="utf-8"))
    out = [changelog.section_body(sections, version).strip()]
    if previous:
        skipped = [
            found
            for found, *_ in sections
            if changelog.version_key(previous.removeprefix("v"))
            < changelog.version_key(found)
            < changelog.version_key(version)
        ]
        for found in skipped:
            date = next(date for name, date, *_ in sections if name == found)
            body = changelog.section_body(sections, found).strip()
            # Older sections use '###' too; one level down keeps them under this heading.
            body = re.sub(r"^### ", "#### ", body, flags=re.MULTILINE)
            out.append(
                f"### Also in this release: {found} ({date})\n\n"
                f"{found} was tagged but never got a release page, so its changes ship here.\n\n{body}"
            )
    return "\n\n".join(out)


def downloads(version: str, tag: str, repo: str, dist: pathlib.Path) -> str:
    files = {path.name: path.stat().st_size for path in dist.iterdir() if path.is_file()}
    shown: set[str] = set()

    def link(name: str) -> str:
        shown.add(name)
        return f"[`{name}`](https://github.com/{repo}/releases/download/{tag}/{name})"

    out = ["## Downloads", "", "### Desktop and mobile", "", "| Platform | Package | File | Size | Notes |", "|---|---|---|---|---|"]
    for pattern, platform, package, notes in DESKTOP:
        name = pattern.format(v=version)
        if name in files:
            out.append(
                f"| {platform} | {package} | {link(name)} | {human_size(files[name])} | {notes.format(v=version)} |"
            )
    out += [
        "",
        "Arch Linux: `yay -S wingdrive-bin` (stable) or `yay -S wingdrive-nightly` (every release).",
    ]
    if f"WingDrive-{version}-aarch64.dmg" in files:
        out += [
            "",
            "> [!NOTE]",
            "> The macOS disk image is signed ad hoc, not notarized (REL-001). After moving WingDrive to",
            "> Applications, run `xattr -dr com.apple.quarantine /Applications/WingDrive.app` once.",
        ]
    out += ["", "### Server and command line", "", "| Platform | Contents | File | Size |", "|---|---|---|---|"]
    for pattern, platform, contents, _ in SERVER:
        name = pattern.format(v=version)
        if name in files:
            out.append(f"| {platform} | {contents} | {link(name)} | {human_size(files[name])} |")
    channel = "latest" if "-" not in version else version
    out += [
        f"| Docker (amd64, arm64) | Server | `docker pull ghcr.io/{repo}/server:v{version}` | — |",
        "",
        "Each server archive holds `bin/wing-server`, `bin/wing`, the media libraries in `lib/wingdrive`"
        " and the licenses. Run `bin/wing-server --help` after extracting it."
        f" The `{channel}` Docker tag follows {'every stable release' if channel == 'latest' else 'this pre-release'}.",
        "",
        "### Packaging and verification",
        "",
        "| File | For | Contents |",
        "|---|---|---|",
    ]
    for name, audience, contents in PACKAGING:
        if name in files:
            out.append(f"| {link(name)} | {audience} | {contents} |")
    for pattern in SIGNATURES:
        name = pattern.format(v=version)
        if name in files:
            out.append(f"| {link(name)} | Desktop | Updater signature |")
    others = sorted(set(files) - shown)
    if others:
        out += ["", "### Other files", ""] + [f"- {link(name)}" for name in others]
    out += [
        "",
        "## Verification",
        "",
        "```sh",
        f"gh release download {tag} --repo {repo} --pattern SHA256SUMS --pattern '<file>'",
        "sha256sum --check --ignore-missing SHA256SUMS",
        "```",
        "",
        "The AppImage and the Windows installer are signed with the WingDrive updater key; installed"
        " apps check that signature before updating.",
    ]
    return "\n".join(out)


def pull_requests(repo: str, tag: str, previous: str | None, token: str) -> str:
    body = {"tag_name": tag}
    if previous:
        body["previous_tag_name"] = previous
    generated = github(f"/repos/{repo}/releases/generate-notes", token, body)["body"]
    groups: dict[str, list[str]] = {}
    contributors: list[str] = []
    in_new = False
    for line in generated.splitlines():
        if line.startswith("## New Contributors"):
            in_new = True
            continue
        if line.startswith("**Full Changelog**"):
            continue
        if in_new:
            if line.startswith("* "):
                contributors.append(line)
            continue
        match = PR_LINE.match(line.strip())
        if not match:
            continue
        title = match["title"]
        kind = TITLE.match(title)
        heading = "Other changes"
        if kind:
            kind_type = kind["type"]
            if kind_type == "chore" and (kind["scope"] or "").startswith("deps"):
                kind_type = "deps"
            if kind_type == "chore" and kind["scope"] == "release":
                kind_type = "release"
            heading = dict(PR_GROUPS).get(kind_type, "Other changes")
        number = match["url"].rsplit("/", 1)[-1]
        groups.setdefault(heading, []).append(f"- {title} ([#{number}]({match['url']})) by @{match['author']}")
    count = sum(len(items) for items in groups.values())
    out = ["## Pull requests", "", f"<details><summary>{count} pull requests merged since {previous or 'the first release'}</summary>", ""]
    order = list(dict.fromkeys(heading for _, heading in PR_GROUPS)) + ["Other changes"]
    for heading in order:
        if heading in groups:
            out += [f"### {heading}", ""] + groups[heading] + [""]
    out.append("</details>")
    if contributors:
        out += ["", "### New contributors", ""] + contributors
    return "\n".join(out)


def main(argv: list[str]) -> int:
    args = [arg for arg in argv if not arg.startswith("--")]
    offline = "--offline" in argv
    previous = None
    if "--previous" in argv:
        previous = argv[argv.index("--previous") + 1]
        args.remove(previous)
    if len(args) != 4:
        print(__doc__, file=sys.stderr)
        return 2
    tag, dist, repo, output = args[0], pathlib.Path(args[1]), args[2], pathlib.Path(args[3])
    version = tag.removeprefix("v")

    missing = [name for name in required_assets(version) if not (dist / name).is_file()]
    if missing:
        print(f"::error::{tag} is missing release assets: {', '.join(missing)}", file=sys.stderr)
        return 1

    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN", "")
    if previous is None and not offline:
        previous = previous_release(repo, tag, token)
    parts = [changelog_part(version, previous), downloads(version, tag, repo, dist)]
    if not offline:
        parts.append(pull_requests(repo, tag, previous, token))
    compare = f"https://github.com/{repo}/compare/{previous}...{tag}" if previous else f"https://github.com/{repo}/commits/{tag}"
    parts.append(f"**Full changelog**: {compare}")
    output.write_text("\n\n".join(parts) + "\n", encoding="utf-8")
    print(f"Wrote {output} for {tag} (previous release: {previous or 'none'})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
