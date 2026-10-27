---
id: DESK-033
title: Act on system files as administrator
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
tags: [desktop, linux, security]
last_updated: 2026-10-27
---

## Description

Dolphin (polkit admin mode) and Nautilus (`admin://`) can copy, rename or delete files the user
does not own after authenticating. WingDrive fails with a permission error.

## Acceptance Criteria

- [ ] A permission error offers to retry the operation through polkit
