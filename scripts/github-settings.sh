#!/usr/bin/env bash
# Applies the repository settings that live outside git: rulesets, merge options,
# secret scanning extras and the CodeQL setup mode.
#
# The rulesets are versioned in .github/rulesets/*.json, so a change to them goes
# through review like any other file. Running this again updates each ruleset in
# place, matched by name, instead of creating duplicates.
#
# Usage: scripts/github-settings.sh [owner/repo]   (needs `gh auth login` as an admin)
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

repo="${1:-bernardopg/wingdrive}"

step() { printf '\n==> %s\n' "$*"; }

# Advanced setup in .github/workflows/codeql.yml needs default setup off, or
# GitHub rejects its uploads.
step "Disable CodeQL default setup"
gh api -X PATCH "repos/$repo/code-scanning/default-setup" -f state=not-configured > /dev/null

step "Merge options"
gh api -X PATCH "repos/$repo" \
	-F delete_branch_on_merge=true \
	-F allow_update_branch=true \
	-F allow_auto_merge=true > /dev/null

step "Secret scanning extras"
gh api -X PATCH "repos/$repo" --input - > /dev/null <<'JSON'
{
	"security_and_analysis": {
		"secret_scanning": { "status": "enabled" },
		"secret_scanning_push_protection": { "status": "enabled" },
		"secret_scanning_non_provider_patterns": { "status": "enabled" },
		"secret_scanning_validity_checks": { "status": "enabled" }
	}
}
JSON

for file in .github/rulesets/*.json; do
	name=$(jq -r .name "$file")
	id=$(gh api "repos/$repo/rulesets" --jq ".[] | select(.name == \"$name\") | .id")
	if [ -n "$id" ]; then
		step "Update ruleset '$name' ($id)"
		gh api -X PUT "repos/$repo/rulesets/$id" --input "$file" > /dev/null
	else
		step "Create ruleset '$name'"
		gh api -X POST "repos/$repo/rulesets" --input "$file" > /dev/null
	fi
done

step "Rules now active on the default branch"
gh api "repos/$repo/rules/branches/main" --jq '.[].type'
