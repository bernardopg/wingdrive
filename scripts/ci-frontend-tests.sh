#!/usr/bin/env bash
# Runs the critical frontend tests listed in .github/workflows/ci.yml.
#
# The Linux job keeps the list as `bun test` lines. The Windows job and
# scripts/ci-local.sh read that list here, so a test added there runs everywhere.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

grep -oE 'bun test [^ ]+' .github/workflows/ci.yml | while read -r _ _ file; do
	bun test "$file"
done
