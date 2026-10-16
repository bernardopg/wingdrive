#!/usr/bin/env bash
# Prints the CI scope for the current GitHub Actions event, as two output lines:
#
#   scope=<value>        what scripts/ci-scope.sh prints for the changed files
#   verified=<bool>      true when a pull request run already passed on this exact tree
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

case "${GITHUB_EVENT_NAME:-}" in
	pull_request)
		git diff --name-only "$BASE_SHA...$HEAD_SHA" | scripts/ci-scope.sh
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
			echo "scope=full"
		else
			git diff --name-only "$BEFORE_SHA" HEAD | scripts/ci-scope.sh
		fi
		echo "verified=$verified"
		;;
	*)
		echo "scope=full"
		echo "verified=false"
		;;
esac
