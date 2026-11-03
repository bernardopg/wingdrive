#!/usr/bin/env bash
# Runs the Rust tests under cargo-llvm-cov and writes one lcov report.
#
# It runs the tests CI runs (scripts/ci-rust.sh tests) plus the unit and
# integration tests of the CLI, the server and the shared crates, which CI only
# lints. Doctests are not run. The instrumented build lives in
# target/llvm-cov-target, so it never invalidates the normal target/.
#
# Usage: scripts/coverage-rust.sh [output.lcov]   (default: coverage/rust.lcov)
# Needs: cargo-llvm-cov and the llvm-tools-preview rustup component.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out="${1:-coverage/rust.lcov}"
mkdir -p "$(dirname "$out")"

eval "$(cargo llvm-cov show-env --export-prefix)"
cargo llvm-cov clean --workspace

scripts/ci-rust.sh tests

printf '\n==> Test CLI, server and shared crates\n'
cargo test --locked --lib --bins --tests \
	-p wing-cli -p wing-server -p wing-tauri-core -p file-opening \
	-p wing-archive -p wing-utils -p wing-fs-watcher -p wing-task-system -p wing-crypto \
	-p wing-actors -p wing-client -p wing-media-metadata -p wing-images

cargo llvm-cov report --lcov --output-path "$out"
printf '\nRust coverage written to %s\n' "$out"
