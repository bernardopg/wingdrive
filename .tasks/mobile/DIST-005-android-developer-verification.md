---
id: DIST-005
title: "Android developer verification: register com.wingdrive.app and its signing keys"
status: To Do
assignee: bernardopg
parent: DIST-000
priority: Medium
milestone: M5
sprint: S11
tags: [android, distribution, signing, policy]
related_tasks: [DIST-001, DIST-006]
last_updated: 2026-11-28
---

## Description

Since 2026-09-30 certified Android devices in Brazil, Indonesia, Singapore and Thailand block apps
from unregistered developers installed from participating stores; Google extends it to every app,
sideloaded ones included, in 2027. Play registers apps it distributes, but the APKs on GitHub
Releases, Obtainium and F-Droid need the package name and every signing key registered. Without
it, users must use the "advanced flow" with a 24-hour wait.

## Implementation Steps

- [ ] In Play Console > Android developer verification, register `com.wingdrive.app`
- [ ] Add the SHA-256 fingerprint of the release key (same as the Play app signing key after DIST-001)
- [ ] If F-Droid signs with its own key (DIST-006), add the F-Droid key fingerprint, using `assets/adi-registration.properties` if the console asks for proof
- [ ] Recheck the global 2027 date on developer.android.com/developer-verification

## Acceptance Criteria

- [ ] The package name shows as registered with every key that signs a published APK
