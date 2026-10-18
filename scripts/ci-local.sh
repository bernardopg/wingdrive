#!/usr/bin/env bash
# Runs the CI gates locally before a push, reusing the warm target/ directory.
#
# Mirrors .github/workflows/ci.yml: task validation, independence check,
# TypeScript typecheck and critical tests, then rustfmt and the Rust gates.
# scripts/ci-scope.sh picks the Rust gates from the files changed against the
# merge base with origin/main, and scripts/ci-rust.sh runs them, as in CI. A
# change limited to the desktop app lints and tests only the desktop crates.
#
# Usage: scripts/ci-local.sh [--all]   (--all forces every Rust gate)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

step() { printf '\n==> %s\n' "$*"; }

base=$(git merge-base HEAD origin/main 2>/dev/null || echo origin/main)
changed=$( { git diff --name-only "$base"; git diff --name-only; git diff --name-only --cached; } | sort -u)
scope=$(printf '%s\n' "$changed" | scripts/ci-scope.sh | cut -d= -f2)
[ "${1:-}" = "--all" ] && scope=full
ts_changed=$(printf '%s\n' "$changed" | grep -E '^(packages/|apps/|bun\.lock|package\.json)' || true)

step "Task files"
cargo run -q -p task-validator -- validate
./scripts/check-wingdrive-independence.sh

if [ -n "$ts_changed" ] || [ "${1:-}" = "--all" ]; then
	step "TypeScript"
	if printf '%s\n' "$changed" | grep -qE '^packages/wingdrive-(primitives|ai)/'; then
		bun run --cwd packages/wingdrive-primitives build
		bun run --cwd packages/wingdrive-ai build
		git diff --exit-code -- packages/wingdrive-primitives/dist packages/wingdrive-ai/dist \
			|| { echo "Stale dist. Commit the rebuilt dist/."; exit 1; }
	fi
	# expo-router writes typed routes into the ignored apps/mobile/.expo while
	# the dev server runs; a copy from another branch fails tsc, and CI never
	# has one, so typecheck without it and put it back afterwards.
	expo_types=apps/mobile/.expo/types
	if [ -d "$expo_types" ]; then
		mv "$expo_types" "$expo_types.ci-local"
		trap 'mv apps/mobile/.expo/types.ci-local apps/mobile/.expo/types' EXIT
	fi
	bun run typecheck
	if [ -d "$expo_types.ci-local" ]; then
		mv "$expo_types.ci-local" "$expo_types"
		trap - EXIT
	fi
	# Same list as the "Test critical frontend behavior" step in ci.yml.
	grep -oE 'bun test [^ ]+' .github/workflows/ci.yml | while read -r _ _ file; do
		bun test "$file"
	done
fi

if [ "$scope" = "none" ]; then
	step "No Rust-relevant changes; skipping Rust gates (pass --all to force)"
	exit 0
fi

step "rustfmt"
cargo fmt --all -- --check

# shellcheck disable=SC2086 # scope is a list of areas.
scripts/ci-rust.sh $scope

step "All CI gates passed"
