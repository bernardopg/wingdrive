#!/usr/bin/env python3
"""Checks CHANGELOG.md against the WingDrive changelog standard and extracts sections.

Usage:
  changelog.py check                 Validate the whole file (CI runs this).
  changelog.py require <version>     Fail unless <version> has a dated section.
  changelog.py extract <version>     Print the body of <version>'s section.
  changelog.py between <old> <new>   Print the versions after <old> up to <new>, newest first.

The standard is documented in docs/core/releases.mdx, section "Changelog". In short:

  ## 3.7.0 — 2026-11-24

  One paragraph that says what the release is about.

  ### Added
  - Sentence about one change, with its pull request (#123).

Sections from 3.7.0 on use only the subsections in SUBSECTIONS, in that order, each with
at least one bullet. Older sections predate the standard and only need a valid heading.
"""

from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CHANGELOG = ROOT / "CHANGELOG.md"

# Order in which subsections appear. "Highlights" repeats the few changes a user should
# read first; "Internal" holds CI, tests, documentation and dependency work.
SUBSECTIONS = (
    "Highlights",
    "Added",
    "Changed",
    "Fixed",
    "Performance",
    "Security",
    "Deprecated",
    "Removed",
    "Internal",
)
# First version written to the standard; older sections are left as they were.
STANDARD_SINCE = (3, 7, 0)

VERSION = r"\d+\.\d+\.\d+(?:-(?:alpha|beta|rc)\.\d+)?"
HEADING = re.compile(rf"^## (?P<version>{VERSION}) — (?P<date>\d{{4}}-\d{{2}}-\d{{2}})$")
UNRELEASED = "## Unreleased"


def version_key(version: str) -> tuple:
    core, _, pre = version.partition("-")
    major, minor, patch = (int(part) for part in core.split("."))
    if not pre:
        return (major, minor, patch, 1, "", 0)
    label, _, number = pre.partition(".")
    return (major, minor, patch, 0, label, int(number))


def parse(text: str) -> tuple[list[str], list[tuple[str, str, int, list[str]]]]:
    """Returns the problems found and the (version, date, line, body) of each section."""
    problems: list[str] = []
    sections: list[tuple[str, str, int, list[str]]] = []
    lines = text.splitlines()
    if not lines or lines[0] != "# Changelog":
        problems.append("line 1: the file must start with '# Changelog'")
    current: list[str] | None = None
    seen_version = False
    for number, line in enumerate(lines, start=1):
        if line.startswith("## "):
            if line == UNRELEASED:
                if seen_version:
                    problems.append(f"line {number}: '## Unreleased' must come before every version")
                current = []
                continue
            match = HEADING.match(line)
            if not match:
                problems.append(
                    f"line {number}: version heading must read '## X.Y.Z — YYYY-MM-DD' (em dash): {line!r}"
                )
                current = None
                continue
            seen_version = True
            current = []
            sections.append((match["version"], match["date"], number, current))
        elif current is not None:
            current.append(line)
    versions = [version_key(version) for version, *_ in sections]
    for (previous, current_key), (version, _, number, _) in zip(
        zip(versions, versions[1:]), sections[1:]
    ):
        if current_key >= previous:
            problems.append(f"line {number}: {version} is not older than the section above it")
    seen: set[str] = set()
    for version, _, number, _ in sections:
        if version in seen:
            problems.append(f"line {number}: {version} appears twice")
        seen.add(version)
    for version, _, number, body in sections:
        if version_key(version)[:3] >= STANDARD_SINCE:
            problems.extend(check_standard(version, number, body))
    return problems, sections


def check_standard(version: str, heading_line: int, body: list[str]) -> list[str]:
    problems: list[str] = []
    summary: list[str] = []
    order: list[str] = []
    bullets: dict[str, int] = {}
    current: str | None = None
    for offset, line in enumerate(body, start=1):
        number = heading_line + offset
        if line.startswith("### "):
            name = line[4:].strip()
            if name not in SUBSECTIONS:
                problems.append(
                    f"line {number}: {version}: unknown subsection {name!r}; use one of {', '.join(SUBSECTIONS)}"
                )
            elif name in order:
                problems.append(f"line {number}: {version}: subsection {name!r} appears twice")
            current = name
            order.append(name)
            bullets[name] = 0
        elif line.startswith("#"):
            problems.append(f"line {number}: {version}: only '###' subsections are allowed inside a version")
        elif current is None:
            if line.strip():
                summary.append(line)
        elif line.startswith("- "):
            bullets[current] += 1
        elif line.strip() and not line.startswith("  "):
            problems.append(
                f"line {number}: {version}: text under '### {current}' must be a '- ' bullet or its indented continuation"
            )
    if not summary:
        problems.append(f"line {heading_line}: {version}: add a summary paragraph before the first subsection")
    known = [name for name in order if name in SUBSECTIONS]
    if known != sorted(known, key=SUBSECTIONS.index):
        problems.append(f"line {heading_line}: {version}: subsections must follow the order {', '.join(SUBSECTIONS)}")
    if not order:
        problems.append(f"line {heading_line}: {version}: add at least one subsection")
    for name, count in bullets.items():
        if count == 0:
            problems.append(f"line {heading_line}: {version}: '### {name}' has no bullets")
    return problems


def section_body(sections, version: str) -> str:
    for found, _, _, body in sections:
        if found == version:
            return "\n".join(body).strip() + "\n"
    raise SystemExit(f"CHANGELOG.md has no section for {version}")


def main(argv: list[str]) -> int:
    if not argv:
        print(__doc__, file=sys.stderr)
        return 2
    command, args = argv[0], [arg.removeprefix("v") for arg in argv[1:]]
    problems, sections = parse(CHANGELOG.read_text(encoding="utf-8"))
    if command == "check":
        for problem in problems:
            print(f"CHANGELOG.md:{problem}", file=sys.stderr)
        if problems:
            print("See docs/core/releases.mdx, section 'Changelog'.", file=sys.stderr)
            return 1
        print(f"CHANGELOG.md follows the standard ({len(sections)} versions)")
        return 0
    if problems:
        for problem in problems:
            print(f"CHANGELOG.md:{problem}", file=sys.stderr)
        return 1
    if command == "require" and len(args) == 1:
        section_body(sections, args[0])
        print(f"CHANGELOG.md has a section for {args[0]}")
        return 0
    if command == "extract" and len(args) == 1:
        sys.stdout.write(section_body(sections, args[0]))
        return 0
    if command == "between" and len(args) == 2:
        old, new = version_key(args[0]), version_key(args[1])
        for version, *_ in sections:
            if old < version_key(version) <= new:
                print(version)
        return 0
    print(__doc__, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
