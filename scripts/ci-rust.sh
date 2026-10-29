#!/usr/bin/env bash
# Runs the Rust CI gates for a scope printed by scripts/ci-scope.sh.
#
# CI and scripts/ci-local.sh both call this, so the two cannot drift. A full scope
# lints the workspace and runs every Rust test CI runs. An app scope such as
# `desktop` lints only that app's crates and runs only its tests, which skips the
# test build of wing-core (about 7 minutes on a hosted runner).
#
# Usage: scripts/ci-rust.sh full | <area>...   (areas: desktop server cli mobile)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

step() {
	if [ -n "${GITHUB_ACTIONS:-}" ]; then
		if [ -n "${in_group:-}" ]; then echo "::endgroup::"; fi
		echo "::group::$*"
		in_group=1
	else
		printf '\n==> %s\n' "$*"
	fi
}

[ "$#" -gt 0 ] || { echo "usage: $0 full | <area>..." >&2; exit 2; }

# Tauri's build script and rust-embed need these directories to exist.
mkdir -p apps/tauri/dist apps/web/dist

# Keeping wing-core out of the desktop shell halves its dependency graph, which
# keeps `tauri dev` and the release bundle fast.
check_desktop_shell_deps() {
	step "Desktop shell does not depend on wing-core"
	local tree
	tree=$(cargo tree --locked -p wingdrive -e normal,build --prefix none)
	if grep -q '^wing-core ' <<< "$tree"; then
		echo "error: apps/tauri/src-tauri depends on wing-core. Move what it needs into a crate under crates/." >&2
		cargo tree --locked -p wingdrive -e normal,build -i wing-core >&2
		exit 1
	fi
}

if [ "$1" = "full" ]; then
	check_desktop_shell_deps

	step "clippy (workspace)"
	cargo clippy --workspace --locked -- -D warnings

	# The TypeScript client is generated from Rust types; a stale copy compiles
	# but sends or reads the wrong shapes at runtime.
	step "Generated TypeScript types are current"
	cargo run --locked --bin generate_typescript_types
	if ! git diff --exit-code -- packages/ts-client/src/generated; then
		echo "error: packages/ts-client/src/generated is out of date. Run 'cargo run --bin generate_typescript_types' and commit the result." >&2
		exit 1
	fi

	# One build of every test binary, so the runs below only execute them.
	step "Build test binaries"
	cargo build --locked --bin wing-daemon
	cargo test --locked --no-run -p wing-core --lib \
		--test branding_legacy_paths \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
	cargo test --locked --no-run -p wingdrive --bin WingDrive
	cargo test --locked --no-run -p wing-branding

	step "Test branding, key storage and legacy path resolution"
	cargo test --locked -p wing-branding
	cargo test --locked -p wing-core --lib crypto::key_manager::tests
	cargo test --locked -p wing-core --lib ops::files::query::file_checksum
	cargo test --locked -p wing-core --lib ops::files::query::vcs_status
	cargo test --locked -p wing-core --lib ops::files::query::templates
	cargo test --locked -p wing-core --lib ops::files::create_entry
	cargo test --locked -p wing-core --test branding_legacy_paths

	step "Test desktop IPC and launch handling"
	cargo test --locked -p wingdrive --bin WingDrive
	cargo test --locked -p file-opening-linux

	step "Test two-device sync"
	cargo test --locked -p wing-core \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
else
	packages=()
	for area in "$@"; do
		read -r -a flags <<< "$(scripts/ci-scope.sh --packages "$area")"
		packages+=("${flags[@]}")
	done

	step "clippy ($*)"
	cargo clippy --locked "${packages[@]}" -- -D warnings

	if [[ " $* " == *" desktop "* ]]; then
		check_desktop_shell_deps

		step "Test desktop IPC and launch handling"
		cargo build --locked --bin wing-daemon
		cargo test --locked -p wingdrive --bin WingDrive
		cargo test --locked -p file-opening-linux
	fi
fi

if [ -n "${in_group:-}" ]; then echo "::endgroup::"; fi
printf '\nRust gates passed (%s)\n' "$*"
