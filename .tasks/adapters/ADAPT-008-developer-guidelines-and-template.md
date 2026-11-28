---
id: ADAPT-008
title: "Developer guidelines, template repository and `wing adapter` CLI"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S06
tags: [adapters, store, docs, cli]
depends_on: [ADAPT-003]
related_tasks: [ADAPT-007]
last_updated: 2026-11-24
---

## Description

Third parties need one path to publish an adapter: start from a template, follow written rules,
check locally what the bots will check, and open a pull request to the registry.

## Implementation Steps

- [ ] Two templates in the registry repository, `templates/python` and `templates/rust`, each with manifest, fixtures, tests, a CI workflow that runs the harness, an icon placeholder and a README; `wing adapter init --lang python|rust` copies one into a new folder or repository
- [ ] `wing adapter init|validate|test|pack` (validate runs the same checks as the bots)
      (2026-11-28: `test` and `validate` exist; `validate` runs the registry's `wingreg check`, see
      ADAPT-007. Left: `init` and `pack`.)
- [ ] Guidelines in `docs/core/adapter-guidelines.mdx` and on wingdrive.org: naming, privacy (read only, no telemetry, local first), permissions, secrets, tests, review rules and reasons for rejection
- [ ] Submission flow: issue form or pull request template in the registry with the checklist

## Acceptance Criteria

- [x] An adapter created from the template passes `wing adapter validate` and the bots without edits
      (2026-11-28: a copy of `templates/python` from registry `main` passes Structure, Code scan,
      Adapter tests and Contract harness under `wing adapter validate`, after the template gained its
      `LICENSE` in bernardopg/wingdrive-adapters#15; the same copy without `LICENSE` and with an
      undeclared host fails. The registry's tests check the template the same way on every pull
      request.)
- [ ] Guidelines reviewed against the bot checks so every rule is either automated or marked as a human check
