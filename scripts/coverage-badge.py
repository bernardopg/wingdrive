#!/usr/bin/env python3
"""Merge lcov reports and write a shields.io endpoint badge.

Usage: coverage-badge.py <label> <output.json> <lcov.info>...

A line counts as covered when any report hit it. Test files and dependencies
are left out. The badge reports the share of instrumented lines that ran.
"""
import json
import sys
from pathlib import Path


def merge(paths):
	lines: dict[str, dict[int, int]] = {}
	for path in paths:
		current = None
		for raw in Path(path).read_text().splitlines():
			if raw.startswith("SF:"):
				current = raw[3:]
				if "node_modules" in current or ".test." in current:
					current = None
				else:
					lines.setdefault(current, {})
			elif raw.startswith("DA:") and current:
				number, hits = raw[3:].split(",")[:2]
				counts = lines[current]
				counts[int(number)] = max(counts.get(int(number), 0), int(hits))
	return lines


def color(percent):
	for limit, name in ((80, "brightgreen"), (60, "green"), (40, "yellow"), (20, "orange")):
		if percent >= limit:
			return name
	return "red"


def main():
	label, output, *reports = sys.argv[1:]
	lines = merge(reports)
	found = sum(len(counts) for counts in lines.values())
	hit = sum(1 for counts in lines.values() for hits in counts.values() if hits > 0)
	percent = 100 * hit / found if found else 0
	badge = {"schemaVersion": 1, "label": label, "message": f"{percent:.0f}%", "color": color(percent)}
	Path(output).write_text(json.dumps(badge) + "\n")
	print(f"{label}: {hit}/{found} lines in {len(lines)} files ({percent:.1f}%)")


if __name__ == "__main__":
	main()
