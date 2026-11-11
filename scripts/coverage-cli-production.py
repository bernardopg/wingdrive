#!/usr/bin/env python3
"""Summarize CLI production lines from an LCOV report.

Use the same line aggregation as coverage-report.py, then exclude the CLI's
inline #[cfg(test)] modules and its separate parameterized test matrices. This
is a focused diagnostic, not the full multi-suite CI coverage report.
"""

import importlib.util
import json
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("coverage_report", SCRIPT / "coverage-report.py")
report = importlib.util.module_from_spec(spec)
spec.loader.exec_module(report)


def inline_test_lines(path: Path) -> set[int]:
    lines = path.read_text().splitlines()
    excluded = set()
    for index, line in enumerate(lines):
        if line != "#[cfg(test)]" or index + 1 >= len(lines):
            continue
        following = lines[index + 1]
        if not (following.startswith("mod ") and following.rstrip().endswith("{")):
            continue
        end = next((i for i in range(index + 2, len(lines)) if lines[i] == "}"), None)
        if end is None:
            raise ValueError(f"Unclosed top-level test module in {path}:{index + 1}")
        excluded.update(range(index + 1, end + 2))
    return excluded


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: coverage-cli-production.py <report.lcov>")
    files = {}
    for path, counts in report.merge([sys.argv[1]]).items():
        if not path.startswith("apps/cli/") or path.endswith("/args/matrix.rs"):
            continue
        ignored = inline_test_lines(report.ROOT / path)
        files[path] = {line: hits for line, hits in counts.items() if line not in ignored}
    print(json.dumps(report.stats(files), indent=2))


if __name__ == "__main__":
    main()
