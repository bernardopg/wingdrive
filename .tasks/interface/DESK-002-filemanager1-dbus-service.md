---
id: DESK-002
title: org.freedesktop.FileManager1 D-Bus service
status: To Do
assignee: bernardopg
parent: DESK-000
priority: High
milestone: M2
sprint: S03
tags: [desktop, linux, dbus]
last_updated: 2026-10-07
---

## Description

Browsers, editors and portals call `org.freedesktop.FileManager1.ShowItems` to reveal a file. WingDrive only calls that interface. It must own the name while running and implement `ShowFolders`, `ShowItems` and `ShowItemProperties`.

## Implementation Steps

- [ ] Own `org.freedesktop.FileManager1` on the session bus with `zbus` when the name is free
- [ ] Implement `ShowFolders`, `ShowItems` (select) and `ShowItemProperties` (open Inspector)
- [ ] Reveal from WingDrive itself keeps working when it owns the name
- [ ] D-Bus activation file so the call starts WingDrive when it is not running

## Acceptance Criteria

- [ ] `gdbus call ... ShowItems ['file:///etc/hosts'] ''` opens /etc with hosts selected
- [ ] "Show in folder" from Firefox opens WingDrive
