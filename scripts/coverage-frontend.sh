#!/usr/bin/env bash
# Collects the shared frontend and mobile CI suites with coverage.
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
mobile_out="$(realpath "$out")/mobile"
(cd apps/mobile && bun test --coverage --coverage-reporter=lcov --coverage-dir="$mobile_out" ./src)
# Bun writes paths relative to the directory that ran the suite.
sed -i 's|^SF:src/|SF:apps/mobile/src/|' "$mobile_out/lcov.info"
printf '\nFrontend coverage for %d shared test files and the mobile suite written to %s\n' "$i" "$out"
