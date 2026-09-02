#!/usr/bin/env bash

set -euo pipefail

readonly forbidden='https?://(www\.)?spacedrive\.com|https?://(releases|models)\.spacedrive\.com|github\.com/spacedriveapp|ghcr\.io/spacedriveapp|@spacedriveapp|@spacedrive/'
readonly operational_identity='com\.spacedrive\.(daemon|core|photos)|spacedrive-daemon|Library/Application Support/spacedrive|spacedrive-(server|data)'

matches="$({
	git grep -nEI "$forbidden" -- \
		':!NOTICE.md' \
		':!docs/overview/history.mdx' \
		':!whitepaper/**' \
		':!plans/wingdrive-independence-migration.md' \
		':!.tasks/core/FORK-001-data-directory-rebrand.md' \
		':!.tasks/core/FORK-002-wingdrive-independence.md' \
		':!**/Cargo.lock' || true
} | grep -vE '^README\.md:[0-9]+:> \*\*WingDrive is a community continuation fork of \[Spacedrive\]' || true)"

if [[ -n "$matches" ]]; then
	echo "Active Spacedrive-owned endpoints remain:" >&2
	echo "$matches" >&2
	exit 1
fi

legacy_matches="$({
	git grep -nEI "$operational_identity" -- \
		':!core/src/branding.rs' \
		':!core/src/ops/locations/import/action.rs' \
		':!core/tests/location_export_import_test.rs' \
		':!apps/cli/src/domains/daemon/mod.rs' \
		':!apps/tauri/src-tauri/src/main.rs' \
		':!packages/swift-client/Tests/SpacedriveClientTests/SerializationTests.swift' \
		':!docs/core/design/archive.md' \
		':!plans/wingdrive-independence-migration.md' \
		':!.tasks/core/FORK-002-wingdrive-independence.md' \
		':!scripts/check-wingdrive-independence.sh' || true
} )"

if [[ -n "$legacy_matches" ]]; then
	echo "Active legacy product identifiers remain:" >&2
	echo "$legacy_matches" >&2
	exit 1
fi

echo "WingDrive independence check passed."
