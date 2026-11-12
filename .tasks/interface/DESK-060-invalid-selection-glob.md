---
id: DESK-060
title: Handle invalid selection glob ranges
status: Done
assignee: bernardopg
priority: Medium
milestone: M3
tags: [desktop, explorer, bug]
---

## Description

Entering `[z-a]` in Select by Pattern or the folder filter throws a regular expression error. An invalid range must not interrupt the Explorer.

## Steps to reproduce

1. Open an Explorer folder and enter `[z-a]` in the selection pattern or folder filter.
2. The generated regular expression throws because the range is reversed.

## Implementation Steps

- [x] Treat invalid character classes as literal text while preserving other patterns.
- [x] Cover invalid ranges and valid alternatives in `selectionPattern.test.ts`.

## Acceptance Criteria

- Invalid ranges do not throw and can match literal names.
- Valid alternatives in the same pattern continue to match.
