---
id: TAURI-001
title: Remove False Desktop Affordances
status: Done
assignee: unassigned
parent: TAURI-000
priority: High
tags: [tauri, interface, bug]
last_updated: 2026-08-24
---

## Description

Remove controls that accepted an action but only changed local state or logged to the console.

## Acceptance Criteria

- [x] Quick Look opens the existing Quick Preview
- [x] Sources search filters real source records
- [x] External file drops are no longer swallowed by the Explorer
- [x] Favorite is hidden until persistence exists
- [x] Delete Sidecar is hidden until a deletion mutation exists
- [x] Install Adapter is hidden until directory installation exists
- [x] Unused Overview mock datasets and components are removed
