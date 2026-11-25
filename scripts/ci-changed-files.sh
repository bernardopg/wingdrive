#!/usr/bin/env bash
# Prints the files a git range changes, one per line, leaving out version files whose only
# change is the product version.
#
# `cargo xtask bump` edits the version in the manifests and Cargo.lock and nothing else, so
# a release pull request would otherwise run every Rust gate, the Explorer smoke, the Android
# build and the Windows installers again on code the base already passed. The release
# workflow builds and verifies everything from the tag anyway.
#
# A version file stays in the list when any changed line is not a `version` line, when the
# added or removed lines carry more than one version, or when Cargo.lock changes a checksum
# or a source (a dependency update).
#
# Usage: scripts/ci-changed-files.sh <range>   (base...head for a pull request, before..HEAD for a push)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

range=$1
version_files='^(Cargo\.lock|core/Cargo\.toml|apps/(cli|server)/Cargo\.toml|apps/tauri/(src-tauri/Cargo\.toml|src-tauri/tauri\.conf\.json|wing-tauri-core/Cargo\.toml|package\.json)|apps/mobile/(app|package)\.json)$'
version_line='^[+-][[:space:]]*"?version"?[[:space:]]*[:=][[:space:]]*"[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?",?$'

# Only the version moved: every changed line is a version line, and the removed and the
# added lines each hold a single version.
only_version_changed() {
	local lines
	lines=$(git diff -U0 --text "$range" -- "$1" | grep -E '^[+-]' | grep -vE '^(\+\+\+|---) ' || true)
	[ -n "$lines" ] || return 1
	grep -qvE "$version_line" <<< "$lines" && return 1
	[ "$(grep '^-' <<< "$lines" | grep -oE '"[0-9][^"]*"' | sort -u | wc -l)" -le 1 ] || return 1
	[ "$(grep '^+' <<< "$lines" | grep -oE '"[0-9][^"]*"' | sort -u | wc -l)" -le 1 ] || return 1
}

# Fails here, not with an empty list, when the range cannot be diffed (a force push).
changed=$(git diff --name-only "$range")

while IFS= read -r file; do
	[ -n "$file" ] || continue
	if [[ "$file" =~ $version_files ]] && only_version_changed "$file"; then
		continue
	fi
	printf '%s\n' "$file"
done <<< "$changed"
