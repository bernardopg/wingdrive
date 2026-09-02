# WingDrive Independence Migration

Status: Local implementation complete; GitHub verification pending  
Owner: bernardopg  
Task: FORK-002  
Updated: 2026-09-02

## Goal

Operate WingDrive without relying on Spacedrive-owned runners, registries,
release endpoints, support channels, package metadata, or product identity.
Keep Spacedrive references only where the license, attribution, history, or
backward compatibility requires them.

## Baseline Audit

The repository contains seven workflow files. Only `wingdrive-ci.yml` runs on
pushes and pull requests. The other six are manual-only inherited definitions,
and several still target Blacksmith or unavailable self-hosted runners.

| Workflow | Current state | Migration decision |
| --- | --- | --- |
| `cache-factory.yaml` | Disabled job tied to `spacedriveapp/spacedrive` | Delete. Normal CI caches cover the fork. |
| `ci.yml` | Manual-only, Blacksmith runners | Replace with the active WingDrive CI and GitHub-hosted runners. |
| `core_tests.yml` | Manual-only, mixed self-hosted and Blacksmith runners | Fold useful checks into CI. |
| `wingdrive-ci.yml` | Active, GitHub-hosted Linux runner | Fold into `ci.yml` and remove the duplicate file. |
| `release.yml` | Manual-only, Blacksmith and signing assumptions | Replace with GitHub-hosted artifact and release jobs. |
| `server.yml` | Manual-only, Blacksmith and inherited tooling | Replace with GitHub-hosted Buildx publishing to this repository's GHCR namespace. |
| `mobile.yml` | Manual-only, inherited bundle IDs, signing team, and runners | Rebrand and split validation from optional store publication. |

Live repository checks on 2026-09-01 found no Actions secrets, no Actions
variables, no branch protection, and no rulesets. Default workflow permissions
are read-only. Release jobs must therefore request only their required
permissions and must not assume signing credentials exist.

Active upstream dependencies found during the audit include repository and
updater URLs, native dependency downloads, a PDF rendering Git dependency,
mobile bundle IDs, support links, extension model URLs, package metadata, and
documentation examples. These are migration targets, not attribution.

## Compatibility Boundary

The following legacy names remain until a separately authorized data
contraction removes them:

- `.spacedrive`, `Spacedrive/Libraries`, and platform config directories used by existing installs
- `Spacedrive` keyring service reads used to adopt existing device keys
- `com.spacedrive` identifiers used only to locate legacy data
- stable internal wire names, database values, MIME types, and crate names when renaming would break compatibility without user benefit
- copyright, license, NOTICE, fork history, and explicit upstream credit

No migration step deletes or moves user data. WingDrive paths win when both
new and legacy paths exist.

## Execution

### 1. Repository automation

- Consolidate pull request and push checks in `.github/workflows/ci.yml`.
- Pin third-party actions to immutable commit SHAs.
- Use least-privilege permissions, concurrency cancellation, timeouts, locked installs, and GitHub-hosted runners.
- Publish server images under `ghcr.io/${{ github.repository }}/server`.
- Build release artifacts from `v*` tags and attach them to this repository's release.
- Keep signing and store upload optional until WingDrive-owned credentials are configured.
- Add Dependabot coverage for GitHub Actions and the package ecosystems used here.

### 2. Product identity

- Set all maintained app names, bundle IDs, schemes, window titles, package descriptions, and About links to WingDrive.
- Replace visible Spacedrive text and artwork with WingDrive assets.
- Preserve legacy readers behind names that clearly identify compatibility behavior.

### 3. Operational autonomy

- Replace active `spacedriveapp` repository, release, updater, issue, registry, and download endpoints.
- Replace or vendor build dependencies hosted only by Spacedrive before removing their URLs.
- Disable optional features whose required models or services are not controlled by WingDrive, rather than silently downloading upstream assets.
- Generate release metadata and checksums inside this repository.

### 4. Content and documentation

- Make WingDrive the subject of current guides, setup commands, screenshots, examples, issue templates, and support instructions.
- Move historical Spacedrive material into an explicitly labeled history or attribution context.
- Keep the FSL license and original copyright statements unchanged.

### 5. Guard and verification

- Fail CI when a new active `spacedriveapp` or `spacedrive.com` dependency appears outside the reviewed allowlist.
- Validate all workflow YAML and action metadata.
- Run task validation, formatting, type checks, focused UI tests, core branding tests, and workspace checks.
- Trigger each manual workflow on a branch before treating release and mobile automation as proven.

## External Setup After Merge

Code cannot create signing identities or store accounts. Configure only the
credentials for channels you intend to publish:

- Apple Developer and App Store Connect credentials for WingDrive bundle IDs
- Android keystore and Google Play service account for the WingDrive package
- Tauri updater signing key if automatic updates are enabled
- Optional repository ruleset requiring the CI jobs after their new names pass on `main`

Unsigned CI artifacts, GitHub Releases, and GHCR publishing must work without
these credentials.

## Completion Evidence

Record workflow run URLs, release URLs, GHCR package tags, and platform artifact
checks in FORK-002. A local YAML parse or successful build is useful evidence,
but does not prove GitHub permissions, publication, signing, or installation.

### Local verification, 2026-09-02

- Workflow syntax passed `actionlint`; workflow and Dependabot YAML parsed.
- Frozen Bun install, maintained frontend typechecks, local UI package builds,
  web production build, and focused frontend tests passed.
- `cargo check --workspace --locked`, branding tests, and the legacy data-path
  integration test passed.
- Expo configuration and a clean Android prebuild generated the WingDrive
  application ID in an isolated workspace copy.
- The independence guard found no active Spacedrive-owned endpoint or package
  dependency outside attribution, history, and compatibility boundaries.
- The WingDrive native dependency release exposes 27 assets, including source
  archives and `SHA256SUMS`; a Linux asset download returned HTTP 200.
- New writers now use WingDrive for daemon services, `wingdrive.json`, location
  export headers, container names, user agents, extension IDs, local storage,
  framework bundle IDs, and active examples. Each reader that must preserve an
  installed base still accepts the reviewed Spacedrive value.
- Removed unused Spacedrive-named placeholder CLI, updater, daemon test, and DEB
  repackaging scripts rather than carrying dead operational paths and an
  external manpage download.
- Final local rerun passed workspace and Tauri Rust checks, CLI and config tests,
  location integration target checking, task validation, workflow lint, shell
  syntax, JSON/YAML parsing, Expo config, frontend typechecks/builds/tests, the
  expanded independence guard, and `git diff --check`.
- A full `sd-core` integration test binary still cannot link locally because of
  the pre-existing `__rust_probestack`/Wasmer native linker conflict. This does
  not affect the passing focused library test or `cargo check` evidence.

### External verification still required

After commit and push, trigger and record terminal GitHub results for `ci.yml`,
`mobile.yml`, `release.yml`, and `server.yml`. Do not enable store publication
or signed updater artifacts until WingDrive-owned signing credentials are
configured.
