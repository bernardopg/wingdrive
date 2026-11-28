---
id: DIST-011
title: "Homebrew: own tap now, official cask after notarization"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Low
milestone: M5
sprint: S10
tags: [macos, homebrew, distribution]
related_tasks: [REL-002]
last_updated: 2026-11-28
---

## Description

Homebrew disabled official casks that fail Gatekeeper on 2026-09-01. The macOS `.dmg` is ad hoc
signed and not notarized, so homebrew/cask would refuse it until REL-002 adds a Developer ID and
notarization (USD 99 a year). A tap owned by the project has no such rule.

## Implementation Steps

- [ ] Create `bernardopg/homebrew-wingdrive` with a `wingdrive` cask for the arm64 `.dmg` and a caveat about the first-run Gatekeeper prompt
- [ ] Update the cask from the release workflow (version and sha256) on stable tags
- [ ] After notarization, submit the cask to homebrew/cask if the repository meets its notability rules

## Acceptance Criteria

- [ ] `brew install --cask bernardopg/wingdrive/wingdrive` installs the latest release
