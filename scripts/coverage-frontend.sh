#!/usr/bin/env bash
# Runs the frontend tests CI runs with coverage and writes one lcov per test file.
#
# CI runs each test file in its own process (module mocks leak between files),
# so this does the same. scripts/coverage-report.py merges the reports.
#
# Usage: scripts/coverage-frontend.sh [output-dir]   (default: coverage/frontend)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out="${1:-coverage/frontend}"
rm -rf "$out"
mkdir -p "$out"

i=0
while read -r _ _ file; do
	i=$((i + 1))
	bun test --coverage --coverage-reporter=lcov --coverage-dir="$out/$i" "$file"
done < <(grep -oE 'bun test [^ ]+' .github/workflows/ci.yml)
printf '\nFrontend coverage for %d test files written to %s\n' "$i" "$out"
