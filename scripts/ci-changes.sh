#!/usr/bin/env bash
# Prints the CI scope for the current GitHub Actions event, as four output lines:
#
#   scope=<value>        what scripts/ci-scope.sh prints for the changed files
#   ts=<bool>            a change reaches TypeScript (packages, apps, Bun manifests)
#   ui=<bool>            a change reaches the web interface the Explorer smoke drives
#   verified=<bool>      true when a pull request run already passed on this exact tree
#
# Changed files come from scripts/ci-changed-files.sh, so a version bump alone
# (`cargo xtask bump`) reaches no gate.
#
# Pull requests are scoped by their own diff. A push to main is scoped by what the
# push added, and is verified when it merged a pull request whose head has the same
# tree and a successful run of $CHECK_WORKFLOW, so main does not repeat that run.
# Schedules and manual runs check everything.
#
# Needs a full clone (fetch-depth: 0), GH_TOKEN, and BASE_SHA/HEAD_SHA on pull requests.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

zero=0000000000000000000000000000000000000000

# Prints the scope, ts and ui lines for the files a range changes.
scope_of() {
	local changed
	changed=$(scripts/ci-changed-files.sh "$1")
	printf '%s\n' "$changed" | scripts/ci-scope.sh
	if grep -qE '^(packages/|apps/|bun\.lock$|package\.json$|scripts/ci-frontend-tests\.sh$|\.github/workflows/ci\.yml$)' <<< "$changed"; then
		echo "ts=true"
	else
		echo "ts=false"
	fi
	if grep -qE '^(packages/|apps/web/|apps/tauri/(src|public)/|bun\.lock$|package\.json$|scripts/release/explorer\.cjs$|\.github/workflows/ci\.yml$)' <<< "$changed"; then
		echo "ui=true"
	else
		echo "ui=false"
	fi
}

everything() {
	echo "scope=full"
	echo "ts=true"
	echo "ui=true"
}

case "${GITHUB_EVENT_NAME:-}" in
	pull_request)
		scope_of "$BASE_SHA...$HEAD_SHA"
		echo "verified=false"
		;;
	push)
		verified=false
		if [ -n "${CHECK_WORKFLOW:-}" ]; then
			pr_head=$(gh api "repos/$GITHUB_REPOSITORY/commits/$GITHUB_SHA/pulls" -q '.[0].head.sha // empty' || true)
			if [ -n "$pr_head" ] \
				&& [ "$(gh api "repos/$GITHUB_REPOSITORY/commits/$pr_head" -q .commit.tree.sha)" = "$(git rev-parse 'HEAD^{tree}')" ]; then
				run=$(gh run list -R "$GITHUB_REPOSITORY" -w "$CHECK_WORKFLOW" --commit "$pr_head" \
					--event pull_request --status success --json databaseId -q '.[0].databaseId // empty')
				if [ -n "$run" ]; then
					echo "Pull request run $run already passed on this tree ($pr_head)." >&2
					verified=true
				fi
			fi
		fi
		if [ -z "${BEFORE_SHA:-}" ] || [ "$BEFORE_SHA" = "$zero" ] || ! git cat-file -e "$BEFORE_SHA^{commit}" 2>/dev/null; then
			everything
		else
			scope_of "$BEFORE_SHA..HEAD"
		fi
		echo "verified=$verified"
		;;
	*)
		everything
		echo "verified=false"
		;;
esac
