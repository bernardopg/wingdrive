---
id: BRAND-002
title: Wingbot WingUI Wingdrop and Wings branding
status: To Do
assignee: bernardopg
parent: UI-000
priority: High
milestone: M1
tags: [branding, interface, compatibility]
last_updated: 2026-10-04
---

## Description

The user selected brand and UI renaming: Wingbot, WingUI, Wingdrop and Wings. Preserve existing API names, persistence, database schemas and external integration compatibility. Do not replace technical terms such as whitespace, namespace, workspace or color space, nor legally required upstream attribution. No changes to repositories outside this workspace.

## Acceptance Criteria

- [ ] User-facing product names and documentation use Wingbot, WingUI, Wingdrop and Wings
- [ ] Owned frontend entry points and imports are renamed consistently where practical
- [ ] Existing RPC, persisted data and legacy URLs remain compatible
- [ ] External package/service identifiers are preserved or adapted explicitly, not blindly replaced
- [ ] Automated checks guard against old visible branding and verify compatibility
- [ ] Typecheck, tests and build pass after the rename

## Validation

Pending implementation after functional work to avoid concurrent renames of worker-owned files.
