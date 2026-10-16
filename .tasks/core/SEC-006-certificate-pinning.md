---
id: SEC-006
title: Certificate Pinning
status: To Do
assignee: jamiepine
parent: SEC-000
priority: Medium
tags: [security, networking, certificate-pinning, upstream-backlog]
whitepaper: Section 8
milestone: M5
last_updated: 2026-10-16
---

## Description

Implement certificate pinning for all connections to third-party cloud storage providers. This will protect against man-in-the-middle attacks by ensuring that the application only connects to trusted servers.

## Implementation Steps

1.  Integrate a library for certificate pinning into the networking stack.
2.  Obtain the public key fingerprints of the trusted cloud storage providers.
3.  Configure the networking stack to only allow connections to servers with a matching public key fingerprint.
4.  Implement a mechanism for updating the pinned certificates.

## Acceptance Criteria

- [ ] The application rejects connections to servers with untrusted certificates.
- [ ] The application can successfully connect to trusted cloud storage providers.
- [ ] The list of pinned certificates can be updated without requiring a full application update.

## WingDrive triage (2026-10-16)

Inherited from upstream Spacedrive. It is not on WingDrive's desktop file-manager path, so it
is parked in the Later milestone M5 (`.tasks/ROADMAP.md`). Re-decide whether WingDrive wants it
before pulling it into a sprint.
