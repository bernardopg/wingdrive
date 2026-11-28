---
id: ADAPT-007
title: "Registry review checks run by Wingbot"
status: Done
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, ci, security, automation]
depends_on: [ADAPT-001, ADAPT-003]
related_tasks: [ADAPT-008, ADAPT-009]
last_updated: 2026-11-28
---

## Description

A pull request that adds or updates an adapter in the registry gets an automatic review before a
human looks at it. The checks run in GitHub Actions and speak as Wingbot (ADAPT-016). The bot checks structure and requirements, runs the adapter's tests and the
contract harness in a sandbox without network, scans for risky code, and comments one report on the
pull request. After a maintainer approves, the bot builds the archive, records its checksum, signs
the index and publishes it.

## Implementation Steps

- [x] Structure: entry schema, matching `adapter.toml`, icon (SVG, square, size limit), summary length, README sections, OSI license, no binaries or vendored dependencies (`wingreg validate`; icons also refuse event handlers, foreignObject, DTDs and external references)
- [x] Requirements: contract version supported, tests present and passing, harness passing against the adapter's fixtures, declared platforms and runtime
      (bernardopg/wingdrive-adapters#13, 2026-11-28: the schema checks contract, platforms and
      runtime; `wingreg validate --run-code` runs each adapter's `tests/` and `wing-adapter-test` on
      its `fixtures/`, both in bubblewrap without network, and every adapter needs a harness case.
      Validate builds `wing-adapter-test` (crate `wing-archive`, #584) from the commit in
      `WINGDRIVE_HARNESS_REF` and caches it by that commit: 2m34s cold, then a cache hit. Building
      the `wing` CLI would compile the whole core, and no release ships the harness yet.)
- [x] Security: declared permissions match what the code uses (network hosts, filesystem paths, secrets); no `eval`/`exec` of fetched code, no shell-outs except declared tools, no writes outside the adapter's state; dependency audit; secret scan
      (`tools/wingreg/checks.py` in #13: hosts in string literals against `permissions.network`,
      network modules without hosts, `~/` paths against `permissions.read`, secret config fields
      against `permissions.secrets`, imports outside the standard library (the dependency audit:
      adapters run with `python3 -I` and no packages), `eval`/`exec`/`pickle.loads`/
      `importlib.import_module`, programs outside `permissions.tools`, committed credentials.
      Writes outside the state folder are not scanned; on Linux the sandbox mounts everything else
      read-only, and the tests and harness run under it in CI.)
- [x] The pinned commit must be reachable from a tag in the adapter repository; updates show the diff since the last pin
      (the tag must point at the pinned commit before anything is fetched; the report links
      `<repository>/compare/<old>...<new>` for a git source whose commit changed, #13)
- [x] Compiled adapters are built by the registry's own CI from the pinned commit for each target (Linux x86_64 and aarch64, macOS aarch64, Windows x86_64), with locked dependencies; binaries uploaded by authors are never accepted, so what users install always matches reviewed source
      (2026-11-28: moved to ADAPT-029, which depends on the Rust template of ADAPT-008. Until it
      lands, `wingreg` refuses every `runtime.kind = "binary"` adapter, so no author binary can
      reach the index.)
- [x] One report comment, updated on every push, with a pass/fail table and links to logs; labels `needs-changes`, `ready-for-review`, `approved` (`wingreg report` + Review comment workflow (`workflow_run`, no pull request code); since #13 the table has Structure, Code scan, Adapter tests and Contract harness per adapter, with the failing output)
- [x] Publish job after approval: archive, checksum, signed index, release asset, website dispatch (website dispatch pending ADAPT-006)
- [x] Same checks runnable locally with `wing adapter validate`
      (2026-11-28: `wing adapter validate [folder]` runs `python -m wingreg check <folder>
      --run-code` (bernardopg/wingdrive-adapters#15) from a shallow clone of the registry in the
      cache folder, or `--registry`, with this same `wing` as the harness. The registry's tooling
      is called instead of ported to Rust: the code scan reads adapter code with Python's `ast`,
      and one implementation keeps the local result and the review the same. A folder outside the
      registry is checked as a community submission, so it needs a `LICENSE`; the Python template
      now ships one. Documented in `docs/core/adapters.mdx`, Validating before a pull request. Run with
      a debug build: a copy of the template passes all four rows; without `LICENSE` and with an
      undeclared host it fails and exits 1.)

## Acceptance Criteria

- [x] A good submission and four bad ones (bad schema, failing tests, undeclared network use, unsigned update) behave as expected in the registry's CI
      (`tests/test_submissions.py` in #13: the template passes all four checks; a bad version,
      a failing test, `urllib.request` with an undeclared host, and an update whose tag does not
      point at the pinned commit each fail with their message. Run 46165091592, "Tooling tests":
      36 passed with the real harness.)
- [x] Publishing an approved submission updates the signed index without manual steps (merge (or `/wingbot publish`) starts Publish; the only manual step is the `registry-publish` approval, by design)
