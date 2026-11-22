---
id: CORE-006
title: Semantic Tagging Architecture
status: Done
assignee: bernardopg
parent: CORE-000
priority: Medium
milestone: M0
tags: [core, vdfs, tagging, metadata]
whitepaper: Section 4.1
---

## Description

Implement the graph-based semantic tagging architecture. This will allow users to create and manage a flexible, hierarchical system of tags for organizing their files.

## Implementation Steps

1.  Design the database schema for storing tags and their relationships.
2.  Implement the logic for creating, renaming, and deleting tags.
3.  Implement the logic for assigning and removing tags from entries.
4.  Develop a query system for finding entries based on their tags.

## Acceptance Criteria

- [x] A user can create and manage a hierarchy of tags. (tag domain/model supports parent
      relationships; legacy checklist reconciled in DEV-008)
- [x] A user can assign multiple tags to a file or directory. (tag application/relationships exist
      in core; legacy checklist reconciled in DEV-008)
- [x] A user can search for files based on their tags. (tag filtering/search exists in core/UI;
      legacy checklist reconciled in DEV-008)
