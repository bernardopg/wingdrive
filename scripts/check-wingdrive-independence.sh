#!/usr/bin/env bash

set -euo pipefail

readonly forbidden='https?://(www\.)?spacedrive\.com|https?://(releases|models)\.spacedrive\.com|github\.com/spacedriveapp|ghcr\.io/spacedriveapp|@spacedriveapp|@spacedrive/'
readonly operational_identity='com\.spacedrive\.(daemon|core|photos)|spacedrive-daemon|Library/Application Support/spacedrive|spacedrive-(server|data)'

matches="$({
	git grep -nEI "$forbidden" -- \
		':!CREDITS.md' \
		':!docs/overview/history.mdx' \
		':!whitepaper/**' \
		':!plans/wingdrive-independence-migration.md' \
		':!.tasks/core/FORK-001-data-directory-rebrand.md' \
		':!.tasks/core/FORK-002-wingdrive-independence.md' \
		':!scripts/check-wingdrive-independence.sh' \
		':!**/Cargo.lock' || true
})"

if [[ -n "$matches" ]]; then
	echo "Active Spacedrive-owned endpoints remain:" >&2
	echo "$matches" >&2
	exit 1
fi

legacy_matches="$({
	git grep -nEI "$operational_identity" -- \
		':!crates/branding/src/lib.rs' \
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

# BRAND-002: old product names must not reach the interface. Only names a
# person reads count here: string literals and JSX text in the frontend and CLI
# output. The code-wide Spacebot check follows.
readonly old_product_names='\b(Spacebot|Spacedrop|SpaceUI)\b'

brand_matches="$({
	git grep -nEI "$old_product_names" -- \
		'packages/interface/src/**' \
		'apps/tauri/src/**' \
		'apps/tauri/src-tauri/src/**' \
		'apps/web/src/**' \
		'apps/cli/src/**' || true
} | grep -vE 'Spacedrop(Send|Args)|spacedrop::|// |//! |/// ' || true)"

if [[ -n "$brand_matches" ]]; then
	echo "Old product names remain in the interface (use Wingbot, Wingdrop, WingUI):" >&2
	echo "$brand_matches" >&2
	exit 1
fi

# Wingbot replaced Spacebot everywhere in code, settings and the RPC API. The
# files listed read names saved before the rename: tab paths, window labels and
# the `spacebot` config key.
wingbot_matches="$({
	git grep -niI 'spacebot' -- \
		'core/**' 'crates/**' 'apps/**' 'packages/**' 'adapters/**' 'scripts/**' \
		':!packages/interface/src/Wingbot/legacyPath.ts' \
		':!packages/interface/src/Wingbot/legacyPath.test.ts' \
		':!packages/interface/src/router.tsx' \
		':!apps/tauri/src/App.tsx' \
		':!core/src/config/app_config.rs' \
		':!scripts/check-wingdrive-independence.sh' || true
})"

if [[ -n "$wingbot_matches" ]]; then
	echo "Spacebot names remain in code (use Wingbot):" >&2
	echo "$wingbot_matches" >&2
	exit 1
fi

echo "WingDrive independence check passed."
