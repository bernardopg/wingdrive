#!/usr/bin/env bash
# Runs the Rust tests under cargo-llvm-cov and writes one lcov report.
#
# It runs the tests CI runs (scripts/ci-rust.sh tests) plus the unit and
# integration tests of the server and the shared crates, which CI only
# lints. Doctests are not run. The instrumented build lives in
# target/llvm-cov-target, so it never invalidates the normal target/.
#
# Usage: scripts/coverage-rust.sh [output.lcov]   (default: coverage/rust.lcov)
# Needs: cargo-llvm-cov and llvm-tools-preview. Linux also needs Xvfb and Bun workspace dependencies.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out="${1:-coverage/rust.lcov}"
mkdir -p "$(dirname "$out")"

if [ "$(uname -s)" = Linux ] && ! command -v Xvfb >/dev/null; then
	printf '%s\n' 'Rust coverage on Linux needs Xvfb for the isolated clipboard test.' >&2
	exit 1
fi

eval "$(cargo llvm-cov show-env --sh)"
cargo llvm-cov clean --workspace

scripts/ci-rust.sh tests

if [ "$(uname -s)" = Linux ]; then
	printf '\n==> Test GTK clipboard on a private X display\n'
	cargo test --locked -p wing-core -p wing-cli -p wing-bench -p wingdrive --bin WingDrive \
		clipboard::tests::clipboard_round_trips_and_falls_back_to_kde_and_plain_uri_lists \
		-- --ignored --exact
fi

printf '\n==> Test server and shared crates\n'
cargo test --locked --lib --bins --tests \
	-p wing-server -p wing-tauri-core -p file-opening \
	-p wing-archive -p wing-utils -p wing-fs-watcher -p wing-task-system -p wing-crypto \
	-p wing-actors -p wing-client -p wing-media-metadata -p wing-images -p wing-elevate

cargo llvm-cov report --lcov --output-path "$out"
printf '\nRust coverage written to %s\n' "$out"
