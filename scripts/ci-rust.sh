#!/usr/bin/env bash
# Runs the Rust CI gates for a scope printed by scripts/ci-scope.sh.
#
# CI and scripts/ci-local.sh both call this, so the two cannot drift. A full scope
# lints the workspace and runs every Rust test CI runs. An app scope such as
# `desktop` lints only that app's crates and runs only its tests, which skips the
# test build of wing-core (about 7 minutes on a hosted runner).
#
# Usage: scripts/ci-rust.sh full | tests | <area>...   (areas: desktop server cli mobile)
#
# `tests` runs only the test part of `full`; scripts/coverage-rust.sh uses it so
# coverage measures the same tests CI runs.
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

# The tests of each platform's file-opening backend compile only on that platform.
case "$(uname -s)" in
	MINGW* | MSYS* | CYGWIN*) file_opening_crate=file-opening-windows ;;
	*) file_opening_crate=file-opening-linux ;;
esac

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

# Core integration suites. The contract suites check pure functions, so they run on every
# platform. The filesystem suites open a library, index or watch real folders, archive or
# move files. They are validated on Linux only, so they stay off the Windows required check
# until a Windows run proves them.
core_contract_tests=(
	addressing_contract_test
	space_contract_test
	tag_validation_contract_test
)
core_filesystem_tests=()
if [ "$(uname -s)" = Linux ]; then
	core_filesystem_tests=(
		archive_contract_test
		tag_manager_contract_test
		indexing_rules_test
		rpc_validation_contract_test
		preflight_test
		search_test
		directory_listing_first_response_test
		ephemeral_coverage_test
		copy_action_test
		location_export_import_test
		event_filtering_test
		job_registration_test
		sync_event_log_test
		tag_integration_test
		tagging_persistence_test
		move_streaming_test
		entry_move_integrity_test
		location_remove_jobs_test
		copy_symlink_test
		copy_conflict_test
		delete_strategy_test
		copy_failure_test
		file_undo_test
		app_config_action_test
		indexing_test
		library_test
		file_queries_test
		google_integration_test
		index_verify_test
		space_layout_test
		file_sync_test
		folder_rename_test
		copy_progress_test
		config_persistence_test
		database_migration_test
		event_system_test
		job_shutdown_test
		normalized_cache_fixtures_test
		indexing_responder_reindex_test
		proxy_pairing_protocol_test
		analyze_events
	)
fi

run_full_tests() {
	# One build of every test binary, so the runs below only execute them.
	step "Build test binaries"
	cargo build --locked --bin wing-daemon
	local core_test_flags=()
	for test in "${core_contract_tests[@]}" ${core_filesystem_tests[@]+"${core_filesystem_tests[@]}"}; do
		core_test_flags+=(--test "$test")
	done
	cargo test --locked --no-run -p wing-core --lib \
		"${core_test_flags[@]}" \
		--test branding_legacy_paths \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
	cargo test --locked --no-run -p wingdrive --bin WingDrive
	cargo test --locked --no-run -p wing-cli --bin wing
	cargo test --locked --no-run -p wing-branding

	step "Test branding, key storage and legacy path resolution"
	cargo test --locked -p wing-branding
	# Every core unit test passed on Linux; other platforms keep the modules
	# already proven there until a run shows the rest pass too.
	if [ "$(uname -s)" = Linux ]; then
		cargo test --locked -p wing-core --lib
	else
		cargo test --locked -p wing-core --lib crypto::key_manager::tests
		cargo test --locked -p wing-core --lib ops::files::query::file_checksum
		cargo test --locked -p wing-core --lib ops::files::query::vcs_status
		cargo test --locked -p wing-core --lib ops::files::query::templates
		cargo test --locked -p wing-core --lib ops::files::create_entry
		cargo test --locked -p wing-core --lib domain::content_identity::tests
	fi
	cargo test --locked -p wing-core --test branding_legacy_paths

	step "Test core contracts"
	for test in "${core_contract_tests[@]}"; do
		cargo test --locked -p wing-core --test "$test"
	done

	if [ "${#core_filesystem_tests[@]}" -gt 0 ]; then
		step "Test core libraries, indexing and file operations on a real filesystem"
		for test in "${core_filesystem_tests[@]}"; do
			case "$test" in
				# These share watcher and indexer timing, so their cases run one at a time.
				# Each case of file_queries_test and google_integration_test starts its own
				# Core, which sets the process-wide current device ID; in parallel, one case
				# could resolve "local" paths against another case's device and find nothing.
				indexing_test | library_test | file_queries_test | google_integration_test)
					cargo test --locked -p wing-core --test "$test" -- --test-threads=1 ;;
				*) cargo test --locked -p wing-core --test "$test" ;;
			esac
		done
	fi

	step "Test CLI contracts and configuration"
	cargo test --locked -p wing-cli --bin wing

	step "Test desktop IPC and launch handling"
	cargo test --locked -p wingdrive --bin WingDrive
	cargo test --locked -p "$file_opening_crate"
	cargo test --locked -p file-clipboard-windows

	step "Test developer tools"
	cargo test --locked -p wing-bench --lib

	step "Test two-device sync"
	cargo test --locked -p wing-core \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
}

if [ "$1" = "tests" ]; then
	run_full_tests
elif [ "$1" = "full" ]; then
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

	run_full_tests
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
		cargo test --locked -p "$file_opening_crate"
		cargo test --locked -p file-clipboard-windows
	fi

	if [[ " $* " == *" cli "* ]]; then
		step "Test CLI contracts and configuration"
		cargo test --locked -p wing-cli --bin wing
	fi
fi

if [ -n "${in_group:-}" ]; then echo "::endgroup::"; fi
printf '\nRust gates passed (%s)\n' "$*"
