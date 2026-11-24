---
id: ADAPT-002
title: "Python runtime for adapters on Windows"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: Medium
milestone: M5
sprint: S09
tags: [adapters, windows, runtime]
related_tasks: [ADAPT-004]
last_updated: 2026-11-24
---

## Description

Python adapters need an interpreter. Windows has none by default, so the store hides Python
adapters there; compiled adapters (ADAPT-003) work without one. Choose between shipping an
embeddable Python with the installer and finding a user-installed interpreter (`py -3`), and
implement it.

## Implementation Steps

- [ ] Measure installer size with the embeddable Python
- [ ] Decide and record the choice in this task
- [ ] Implement the chosen runtime and the interpreter lookup

## Acceptance Criteria

- [ ] An adapter installed from the store syncs on a real Windows machine
