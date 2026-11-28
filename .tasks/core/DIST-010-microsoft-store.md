---
id: DIST-010
title: "Microsoft Store: publish the signed Win32 installer"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S11
tags: [windows, microsoft-store, distribution, signing, blocked]
related_tasks: [REL-002, DIST-009]
last_updated: 2026-11-28
---

## Description

Individual developer accounts are free. The Store accepts an `.exe` or `.msi` only when the
installer and every PE file in it are Authenticode-signed with a certificate from a CA in the
Microsoft Trusted Root Program, so this waits on REL-002. Azure Artifact Signing is open to
individuals only in the USA and Canada, so an individual in Brazil needs an OV certificate from a
CA (Certum, Sectigo, SSL.com). MSIX would get free Store signing but needs repackaging the daemon,
the tray and the UAC helper, which is more work.

## Implementation Steps

- [ ] Wait for signed Windows installers (REL-002)
- [ ] Register an individual account at storedeveloper.microsoft.com
- [ ] Reserve the name WingDrive and create an MSI/EXE app with the versioned GitHub Release URL of the signed NSIS installer and its silent switch `/S`
- [ ] Listing in en-US and pt-BR: description, 1366x768 or larger screenshots, privacy policy URL, IARC rating
- [ ] Each release: update the installer URL in Partner Center, or through the Microsoft Store submission API from the release workflow

## Acceptance Criteria

- [ ] WingDrive installs from the Microsoft Store and its updater keeps working
