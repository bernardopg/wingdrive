#!/usr/bin/env bash
# Prints which Rust gates a change needs, from a list of changed paths on stdin.
#
# CI (.github/workflows/ci.yml) and scripts/ci-local.sh share this filter so both
# skip the same work. Output is one line, `scope=<value>`:
#
#   none                 no changed file reaches a Rust compile
#   full                 core, shared crates, lockfile, toolchain or CI changed
#   desktop server ...   only these app crates changed (desktop, server, cli, mobile)
#
# Pass a scope to `--packages <area>` to get the cargo `-p` flags for that area.
set -euo pipefail

if [ "${1:-}" = "--packages" ]; then
	case "${2:-}" in
		desktop) echo "-p wingdrive -p wing-tauri-core -p file-opening -p file-opening-linux -p file-opening-macos -p file-opening-windows -p wing-desktop-macos" ;;
		server) echo "-p wing-server" ;;
		cli) echo "-p wing-cli" ;;
		mobile) echo "-p wing-mobile-core" ;;
		*) echo "unknown area: ${2:-}" >&2; exit 2 ;;
	esac
	exit 0
fi

# Files that never reach a Rust compile: docs, task tracking, the TypeScript
# packages and app frontends, and release-only scripts.
changed=$(cat)
rust_files=$(printf '%s\n' "$changed" \
	| grep -vE '^(\.tasks/|docs/|\.claude/|packages/|apps/web/|apps/tauri/(src|public)/|apps/tauri/[^/]+$|scripts/release/)' \
	| grep -vE '^apps/mobile/' \
	| grep -vE '\.(md|mdx)$|^(LICENSE|bun\.lock|package\.json)$' \
	| grep -v '^$' || true)
rust_files+=$'\n'$(printf '%s\n' "$changed" | grep -E '^apps/mobile/modules/' || true)

areas=()
add_area() {
	local area
	for area in "${areas[@]}"; do [ "$area" = "$1" ] && return; done
	areas+=("$1")
}

while IFS= read -r file; do
	case "$file" in
		"") ;;
		apps/tauri/*) add_area desktop ;;
		apps/server/*) add_area server ;;
		apps/cli/*) add_area cli ;;
		apps/mobile/modules/*) add_area mobile ;;
		# Anything else (core, crates, Cargo.lock, toolchain, workflows) can break every binary.
		*) echo "scope=full"; exit 0 ;;
	esac
done <<< "$rust_files"

if [ "${#areas[@]}" -eq 0 ]; then
	echo "scope=none"
else
	echo "scope=${areas[*]}"
fi
