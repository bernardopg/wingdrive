---
id: INTG-002
title: Privacy policy, terms and Limited Use disclosure on wingdrive.org
status: To Do
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S06
tags: [integrations, google, website, legal]
last_updated: 2026-11-12
related_tasks: [INTG-001]
---

## Description

Google's OAuth consent screen and app verification require a public privacy policy and terms of service on a verified domain, and apps that use Google user data must state compliance with the Google API Services User Data Policy, including the Limited Use requirements.

## Implementation Steps

1. Write `/privacy` and `/terms` pages in `website/` (EN and PT): what WingDrive accesses (Drive files, Photos the user uploads or picks, profile email), that data stays on the user's devices, that nothing is sent to WingDrive servers, how to revoke access.
2. Add the Limited Use disclosure sentence required by Google.
3. Verify `wingdrive.org` in Google Search Console with the same Google account that owns the Cloud project.
4. Link the pages from the website footer and from Settings > About.

## Acceptance Criteria

- [ ] https://wingdrive.org/privacy and /terms are live in both languages.
- [ ] The domain shows as verified in Google Cloud Console.
