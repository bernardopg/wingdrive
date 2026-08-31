---
id: SAFE-002
title: "Block recursive copies into their source tree"
status: "Done"
assignee: "jamiepine"
priority: "High"
tags: [core, filesystem, safety, copy]
---

## Description

Copying a directory into itself or one of its descendants lets the recursive walker discover its own output. The job then expands until the target volume is full.

## Implementation Steps

- [x] Resolve destinations through existing ancestors and symlinks
- [x] Validate the final per-source destination in the action builder and after job path resolution
- [x] Verify focused tests and workspace compilation

## Acceptance Criteria

- Direct and symlink-mediated destinations inside a source directory are rejected
- Copying a directory to a sibling remains valid
- Jobs created without the action builder receive the same protection after content paths resolve
