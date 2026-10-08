#!/usr/bin/env bash
# Runs the CI gates locally before a push, reusing the warm target/ directory.
#
# Mirrors .github/workflows/ci.yml: task validation, independence check,
# TypeScript typecheck and critical tests, then rustfmt, clippy and the Rust
# tests CI runs. Rust steps are skipped when nothing Rust-relevant changed
# against the merge base with origin/main, with the same filter CI uses.
#
# Usage: scripts/ci-local.sh [--all]   (--all forces the Rust steps)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

step() { printf '\n==> %s\n' "$*"; }

base=$(git merge-base HEAD origin/main 2>/dev/null || echo origin/main)
changed=$( { git diff --name-only "$base"; git diff --name-only; git diff --name-only --cached; } | sort -u)
rust_changed=$(printf '%s\n' "$changed" \
	| grep -vE '^(\.tasks/|docs/|\.claude/|packages/|apps/web/|apps/tauri/(src|public)/|apps/tauri/[^/]+$|scripts/release/)' \
	| grep -vE '^apps/mobile/' \
	| grep -vE '\.(md|mdx)$|^(LICENSE|bun\.lock|package\.json)$' || true)
rust_changed+=$(printf '%s\n' "$changed" | grep -E '^apps/mobile/modules/' || true)
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
	bun run typecheck
	# Same list as the "Test critical frontend behavior" step in ci.yml.
	grep -oE 'bun test [^ ]+' .github/workflows/ci.yml | while read -r _ _ file; do
		bun test "$file"
	done
fi

if [ -z "$rust_changed" ] && [ "${1:-}" != "--all" ]; then
	step "No Rust-relevant changes; skipping Rust gates (pass --all to force)"
	exit 0
fi

mkdir -p apps/tauri/dist apps/web/dist

step "rustfmt"
cargo fmt --all -- --check

step "clippy"
cargo clippy --workspace --locked -- -D warnings

step "Rust tests"
cargo build --locked --bin wing-daemon
cargo test --locked -p wing-core --lib branding::
cargo test --locked -p wing-core --lib crypto::key_manager::tests
cargo test --locked -p wing-core --test branding_legacy_paths
cargo test --locked -p wingdrive --bin WingDrive
cargo test --locked -p wing-core --test sync_metrics_test --test sync_realtime_test --test sync_backfill_test

step "All CI gates passed"
