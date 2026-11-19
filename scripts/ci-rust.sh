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
	wing_contract_test
	tag_validation_contract_test
)
core_filesystem_tests=()
if [ "$(uname -s)" = Linux ]; then
	core_filesystem_tests=(
		archive_contract_test
		aggregate_sizes_watcher_test
		google_drive_picker_test
		microsoft_integration_test
		uuid_reconciliation_test
		volume_offline_test
		tag_manager_contract_test
		indexing_rules_test
		rpc_validation_contract_test
		preflight_test
		search_test
		directory_listing_first_response_test
		ephemeral_coverage_test
		copy_action_test
		duplicate_detection_test
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
		wing_layout_test
		file_sync_test
		file_sync_simple_test
		sync_backfill_race_test
		folder_rename_test
		copy_progress_test
		config_persistence_test
		database_migration_test
		event_system_test
		job_shutdown_test
		ephemeral_bridge_test
		typescript_bridge_test
		typescript_search_bridge_test
		own_data_location_test
		normalized_cache_fixtures_test
		indexing_responder_reindex_test
		proxy_pairing_protocol_test
		analyze_events
	)
fi

# Every test command below builds the same packages, so Cargo resolves one feature set
# and compiles each dependency once. Narrowing `-p` to one package resolves different
# features for shared crates (wing-core gains `cli` from wing-cli; tokio, windows-sys and
# others differ), and Cargo rebuilds them and everything above them: on a cold Windows
# runner that cost about 35 minutes per extra package set.
test_packages=(-p wing-core -p wing-cli -p wing-bench -p wingdrive)
cargo_test() {
	cargo test --locked "${test_packages[@]}" "$@"
}

# Core unit tests that run on Windows and macOS. Every core unit test passed on Linux; other
# platforms keep the modules already proven there until a run shows the rest pass too.
# `recipe::` is wing-bench's only test module, which shares the `--lib` run.
non_linux_lib_tests=(
	crypto::key_manager::tests
	ops::files::query::file_checksum
	ops::files::query::vcs_status
	ops::files::query::templates
	ops::files::create_entry
	domain::content_identity::tests
	recipe::
)

run_full_tests() {
	# One build of every test binary, so the runs below only execute them. Building
	# wing-core's integration tests also builds its binaries (wing-daemon and the type
	# generators) with the same features.
	step "Build test binaries"
	local core_test_flags=()
	for test in "${core_contract_tests[@]}" ${core_filesystem_tests[@]+"${core_filesystem_tests[@]}"}; do
		core_test_flags+=(--test "$test")
	done
	local cli_test_flags=()
	if [ "$(uname -s)" = Linux ]; then
		cli_test_flags+=(--test daemon_service_test)
	fi
	cargo_test --no-run --lib --bin WingDrive --bin wing \
		"${cli_test_flags[@]}" \
		"${core_test_flags[@]}" \
		--test branding_legacy_paths \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
	# Small crates with their own dependency trees; building them apart costs seconds.
	cargo test --locked --no-run -p wing-branding
	cargo test --locked --no-run -p "$file_opening_crate"
	cargo test --locked --no-run -p file-clipboard-windows

	step "Test branding, key storage and legacy path resolution"
	cargo test --locked -p wing-branding
	cargo_test --test branding_legacy_paths

	step "Test core and developer tool units"
	if [ "$(uname -s)" = Linux ]; then
		cargo_test --lib
	else
		cargo_test --lib -- "${non_linux_lib_tests[@]}"
	fi

	step "Test core contracts"
	for test in "${core_contract_tests[@]}"; do
		cargo_test --test "$test"
	done

	if [ "${#core_filesystem_tests[@]}" -gt 0 ]; then
		step "Test core libraries, indexing and file operations on a real filesystem"
		for test in "${core_filesystem_tests[@]}"; do
			case "$test" in
				# These share watcher and indexer timing, so their cases run one at a time.
				# Each case of file_queries_test and google_integration_test starts its own
				# Core, which sets the process-wide current device ID; in parallel, one case
				# could resolve "local" paths against another case's device and find nothing.
				indexing_test | library_test | file_queries_test | google_integration_test | typescript_bridge_test)
					cargo_test --test "$test" -- --test-threads=1 ;;
				*) cargo_test --test "$test" ;;
			esac
		done
	fi

	step "Test CLI contracts and configuration"
	cargo_test --bin wing
	if [ "$(uname -s)" = Linux ]; then
		cargo_test --test daemon_service_test
	fi

	step "Test desktop IPC and launch handling"
	cargo_test --bin WingDrive
	cargo test --locked -p "$file_opening_crate"
	cargo test --locked -p file-clipboard-windows

	step "Test two-device sync"
	cargo_test \
		--test sync_metrics_test \
		--test sync_realtime_test \
		--test sync_backfill_test
}

# The TypeScript client is generated from Rust types; a stale copy compiles but sends or
# reads the wrong shapes at runtime. It runs the generator the test build already linked,
# so it adds no build of its own. The types do not depend on the platform, so Windows and
# macOS skip it.
check_generated_types() {
	[ "$(uname -s)" = Linux ] || return 0
	step "Generated TypeScript types are current"
	"${CARGO_TARGET_DIR:-target}/debug/generate_typescript_types"
	if ! git diff --exit-code -- packages/ts-client/src/generated; then
		echo "error: packages/ts-client/src/generated is out of date. Run 'cargo run --bin generate_typescript_types' and commit the result." >&2
		exit 1
	fi
}

if [ "$1" = "tests" ]; then
	run_full_tests
elif [ "$1" = "full" ]; then
	check_desktop_shell_deps

	step "clippy (workspace)"
	cargo clippy --workspace --locked -- -D warnings

	run_full_tests
	check_generated_types
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
		if [ "$(uname -s)" = Linux ]; then
			cargo test --locked -p wing-cli --test daemon_service_test
		fi
	fi
fi

if [ -n "${in_group:-}" ]; then echo "::endgroup::"; fi
printf '\nRust gates passed (%s)\n' "$*"
