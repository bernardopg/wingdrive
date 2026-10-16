---
id: CORE-010
title: File Ingestion Workflow
status: To Do
assignee: jamiepine
parent: CORE-000
priority: High
tags: [core, vdfs, ingestion, workflow, upstream-backlog]
whitepaper: Section 4.4.7
milestone: M5
last_updated: 2026-10-16
---

## Description

Implement the "Ingest Location" workflow, which provides a quarantine zone for new file uploads. This will allow for user-configurable processing of new files before they are added to the main library.

## Implementation Steps

1.  Design the concept of an "Ingest Location", a special type of location where new files are placed.
2.  Implement a workflow engine that can apply a series of processing steps to files in the Ingest Location.
3.  Develop a set of default processing steps (e.g., virus scanning, metadata extraction, AI analysis).
4.  Allow users to configure the processing steps for each Ingest Location.

## Acceptance Criteria

- [ ] A user can configure an Ingest Location.
- [ ] New files uploaded to the Ingest Location are processed according to the configured workflow.
- [ ] Processed files are moved to their final destination in the library.

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
