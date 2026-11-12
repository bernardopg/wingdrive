---
id: CORE-020
title: Preserve filesystem errors during conflict planning
status: Done
assignee: bernardopg
parent: CORE-000
priority: High
tags: [core, files, bug]
milestone: M5
last_updated: 2026-11-12
---

## Description

Conflict planning treats an inaccessible target as an available path. The generated-name search also treats metadata errors as available names.

## Implementation Steps

- [x] Reproduce with a target under a directory without search permission.
- [x] Propagate metadata errors other than a missing path.
- [x] Cover both conflict decisions and generated names with a regression test.

## Acceptance Criteria

- Planning reports permission errors instead of proposing an unverified destination.
- Missing paths remain available, and dangling links remain conflicts.

## Verification

`ops::files::plan::tests::inaccessible_target_is_not_treated_as_available` checks both operations against a directory without search permission. `dangling_symlinks_are_conflicts_and_reserve_names` covers links whose targets are missing.
