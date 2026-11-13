---
id: INTG-028
title: MEGA integration
status: To Do
assignee: bernardopg
parent: INTG-000
priority: Low
milestone: M5
sprint: S14
tags: [integrations, mega, encryption]
last_updated: 2026-11-12
related_tasks: [INTG-020]
---

## Description

MEGA encrypts files client side. Its official SDK is C++ (BSD-2), so the connector needs bindings or a Rust implementation of the protocol, and keys never leave the device.

## Implementation Steps

1. Evaluate the C++ SDK through FFI against a Rust client.
2. Email, password and 2FA sign-in; store the session, never the password.
3. Backend with client-side encryption and decryption.

## Acceptance Criteria

- [ ] Users connect the service from Settings > Integrations with no developer setup.
- [ ] Browse, upload, download, rename, trash and cloud sync work against the real service.
- [ ] Several accounts per library.
- [ ] Docs in `docs/core/integrations.mdx` and on the website.
