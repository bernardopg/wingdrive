# CI and release performance review

Date: 2026-11-04. Data: the last 40 runs of each workflow (`gh run list`, `gh run view --json jobs`),
the job logs of the slow runs, `gh cache list` and `.github/rulesets/main.json`. Times are wall clock
minutes. p50 is the median, p90 the 90th percentile.

## Summary

The path from a small Rust fix to a published release takes about 70 minutes when every cache is warm,
and close to two hours when the CI Rust cache is missing. Release 2.4.1 is the warm case: pull request
#316 opened at 16:22, merged at 16:59 (37 min), the tag was pushed at 17:00 and the release was published
at 17:31 (31 min).

Two things set that time:

1. CodeQL `Analyze (rust)` takes 35 to 37 minutes on every pull request that touches `*.rs`, and code
   scanning merge protection waits for it. It is the critical path of every Rust pull request, longer
   than the whole `Rust Workspace` job (14 min).
2. The repository uses 9.1 GB of its 10 GB Actions cache. Release runs fill it with caches that no later
   run can read, so GitHub evicts the CI Rust cache. A `Rust Workspace` run without that cache took
   51.7 minutes instead of 14 (#318, run 41747911141, `No cache found`).

Test coverage does not slow pull requests today: `coverage.yml` runs only on `main` and on a weekly
schedule, and no merge or release waits for it. It does take 2 GB of the cache budget.

## Current timings

### Pull request workflows

| Workflow / job | Runs on | p50 | max | Gates merge |
| --- | --- | --- | --- | --- |
| CI `Detect Rust changes` | every PR | 0.2 | 2.9 | yes (required check) |
| CI `Repository Metadata` | every PR | 0.5 | 3.1 | yes |
| CI `TypeScript` | every PR | 0.8 | 1.4 | yes |
| CI `WASM Extensions` | every PR | 0.8 | 1.5 | no |
| CI `Rust Workspace` (cache hit, full scope) | Rust PRs | 14.2 | 16.1 | yes |
| CI `Rust Workspace` (cache miss) | Rust PRs | 51.7 | 51.7 | yes |
| CodeQL `Analyze (rust)` | PRs with `*.rs` | 35.5 | 36.7 | yes (code scanning rule) |
| CodeQL other languages | every PR | 0.8 to 1.5 | 1.8 | yes (code scanning rule) |
| TruffleHog | every PR | 0.5 | 1.4 | yes |
| Mobile `Android build (Gradle)` | mobile, core, crates, lockfile PRs | 10.2 | 11.2 | no |
| Mobile `Embedded core (clippy, aarch64)` | same | 2.9 | 8.9 | no |

Inside a warm `Rust Workspace` run (about 12 minutes of gates), wing-core compiles three times: clippy
(2 min), the `generate_typescript_types` build (3 min) and the test build. Setup takes 1.5 to 3 minutes.

### Main, coverage and release

| Workflow / job | Trigger | p50 | max |
| --- | --- | --- | --- |
| CI on `main` | push | 0.3 | 17.8 |
| CodeQL on `main` | push | 2.4 | 38.8 |
| Coverage | push to `main` | 17.7 | 43.1 |
| Release (whole run) | tag push | 39.8 | 156.0 |
| Release `Verify Linux x86_64 desktop bundles` | tag | 38.7 | 42.1 |
| Release `Android / Signed Android release` | tag | 26.4 | 28.2 |
| Release `Verify Linux x86_64 server and command line` | tag | 22.4 | 36.7 |
| Release `Require green CI for the tagged commit` | tag | 0.1 | 1.2 |
| Release `Publish` + `AUR` | tag | 0.4 + 1.7 | 3.7 |
| Server Image (amd64 + arm64 in parallel) | tag | 23.0 | 23.0 |

CI on `main` already skips a push that merged a pull request whose head has the same tree
(`scripts/ci-changes.sh`), which is why its p50 is 0.3 minutes.

## Critical path

Frontend-only pull request: CI 1 to 3 minutes, CodeQL uploads an empty Rust result, merge after about
5 minutes.

Rust pull request: `max(CodeQL rust 36, Rust Workspace 14 warm / 52 cold)`, so 36 to 52 minutes.

Merge to release: auto-merge merges on the first green run. The tag can follow at once because the
release's CI gate reuses the pull request's run. The release then waits for the desktop bundle job,
about 31 to 40 minutes, of which `tauri build` is 24 to 37 minutes.

Total for a small Rust fix: about 70 minutes warm, up to 2 hours with a cold CI cache.

## Causes, ranked

### 1. CodeQL Rust on every Rust pull request (36 min, blocks merge)

`build-mode: none` for Rust spends 35 minutes in `Perform CodeQL analysis`. `paths-ignore` already drops
the extra workspaces. There is no build to cache, so only a smaller scope or a different trigger can
shorten it. Because the code scanning rule waits for a result, this job alone sets the merge time of
every Rust change.

### 2. Actions cache eviction (up to +38 min on `Rust Workspace`)

`gh cache list` on 2026-11-04 showed 9.09 GB in 49 entries and no `...-ci-...` Rust cache. What took the
space:

| Cache | Size | Readable by later runs |
| --- | --- | --- |
| Server image buildkit blobs on `refs/tags/v2.4.1` | about 1.8 GB | no: a tag run reads only its own tag and `main` |
| Rust `coverage` on `main` | 1.96 GB | only by `coverage.yml` |
| Rust `release-desktop` on `main` | 1.55 GB | yes, by tag releases |
| Rust `release-server` on `main` | 1.11 GB | yes, by tag releases |
| Gradle on `main` | about 1.6 GB | yes, by mobile jobs |
| Rust `mobile` (aarch64) on `main` | 0.54 GB | yes |

The CI Rust cache is about 2.2 GB. After release 2.4.1 wrote the buildkit blobs, the least recently used
cache was the CI one. The next pull request (#318) compiled everything from scratch.

The buildkit cache never helped: the v2.4.1 image build shows no `CACHED` step, and `cargo build` inside
the Dockerfile ran cold for 1093 seconds. `cache-to: type=gha,mode=max` on a tag writes a cache that only
that same tag can read.

### 3. Release desktop bundle (31 to 40 min)

`release.yml` sets `CARGO_BUILD_JOBS: '2'` on a 4 vCPU runner, so the release build uses half the cores.
It was probably set to avoid running out of memory during linking with thin LTO. The build is the bulk
of the release time.

### 4. Off-path work that still costs runner time

- Mobile `Android build (Gradle)` runs on every core or crates change, but it builds with
  `WING_SKIP_RUST=1`, so Rust changes never affect it. 10 minutes per Rust pull request.
- Mobile `Embedded core (clippy)` runs for JavaScript-only mobile changes, where it checks nothing new.
- Coverage runs on every push to `main`, docs and task files included.

None of these block a merge, but they share the account's concurrent job limit with the jobs that do.

## Changes in this pull request

| Change | File | Saving | Risk |
| --- | --- | --- | --- |
| Drop the gha layer cache of the server image | `server.yml` | frees about 1.8 GB per release, keeps the CI Rust cache, avoids 38 min cold `Rust Workspace` runs | none: the cache had zero hits |
| Stop saving the coverage Rust cache | `coverage.yml` | frees 1.96 GB | coverage Rust job runs cold, about 40 min instead of 18, off every merge and release path |
| Skip coverage on docs, tasks and Markdown pushes | `coverage.yml` | one 18 to 43 min run per such push | none: those files hold no tested code |
| Run the Android Gradle build only for mobile and JS changes, and Android clippy only for Rust changes | `mobile.yml` | 10 min per Rust-only PR, 3 to 9 min per mobile JS PR | low: neither job is a required check; manual runs and unknown diffs still run both |

With these changes the steady cache footprint drops from about 9.1 GB plus the 2.2 GB CI cache (over
the limit) to about 7.5 GB including it.

Every gate stays: fmt, clippy `-D warnings`, the generated types check, typecheck, the frontend and Rust
test lists, TruffleHog and every CodeQL language.

## Decisions applied (2026-11-04)

The author approved the first three recommendations of the review:

1. CodeQL Rust runs on pushes to `main` (when the push reaches Rust code), on the weekly schedule and on
   manual runs. Pull requests skip it and upload the empty result code scanning protection needs, so a
   Rust pull request no longer waits 36 minutes for it. A high severity Rust alert now surfaces on `main`
   right after merge instead of blocking the pull request.
2. `release.yml` sets `CARGO_BUILD_JOBS` to 4, one job per vCPU of the hosted runner. The rust-cache key
   includes `CARGO_*` variables, so the first release after the change builds from a cold release cache.
3. Caches deleted by hand: the 35 buildkit entries of `refs/tags/v2.4.1` (1.84 GB), the coverage Rust
   cache (1.96 GB, no longer saved) and the two release Rust caches keyed on the old job count (2.67 GB).
   The repository went from 8.46 GB in 49 entries to 2.19 GB in 11, all on `main`.

How GitHub keeps caches: an entry not read for 7 days is removed, and when the repository passes 10 GB the
least recently used entries go first. A run reads caches of its own ref and of the default branch, so
caches saved on a tag or a pull request ref only serve later runs of that same ref and sit unused until
they expire or push useful entries out.

## Recommendations still open

1. Merge `generate_typescript_types` into the test build of `scripts/ci-rust.sh` (build it with the same
   features and profile as the test binaries). Saving: up to 3 minutes of every full `Rust Workspace`
   run. Needs a check that the generated output does not depend on the feature set.
2. A merge queue does not help at the current volume: `strict_required_status_checks_policy` is off,
   so pull requests never wait for each other.
3. sccache with the GitHub Actions backend would share the same 10 GB budget, so it does not fix
   eviction. A remote cache (S3, R2) would, at the cost of a secret and a service.

## How #318 failed

The `TypeScript` job's stale `dist/` gate failed because commit `5d25b5cdf` (PR #319) changed
`packages/wingdrive-primitives/src/CircleButton.tsx` without rebuilding the committed `dist/`.
`just ci-local` runs the same check whenever `packages/wingdrive-primitives/` changes, so running it
before the push would have caught the failure in under a minute.
