---
id: INTG-002
title: Privacy policy, terms and Limited Use disclosure on wingdrive.org
status: In Progress
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, website, legal]
last_updated: 2026-11-13
related_tasks: [INTG-001]
---

## Description

Google's OAuth consent screen and app verification require a public privacy policy and terms of service on a verified domain, and apps that use Google user data must state compliance with the Google API Services User Data Policy, including the Limited Use requirements.

## Implementation Steps

1. Write `/privacy` and `/service-terms` pages in `website/` (EN and PT): what WingDrive accesses (Drive files, Photos the user uploads or picks, profile email), that data stays on the user's devices, that nothing is sent to WingDrive servers, how to revoke access.
2. Add the Limited Use disclosure sentence required by Google.
3. Verify `wingdrive.org` in Google Search Console with the same Google account that owns the Cloud project.
4. Link the pages from the website footer and from Settings > About.

## Progress (2026-11-13)

`website/` has `/privacy` and `/service-terms` in English and Portuguese (`LegalPage.astro`,
strings in `src/i18n/strings.ts`), linked from the footer. The privacy policy lists the Google and
Microsoft data WingDrive accesses, how it is used and stored, how to revoke access, and the Limited
Use sentence. The author added the Search Console TXT record for `wingdrive.org` at Hostinger;
DNS propagation pending. A link from Settings > About is still to do.

## Acceptance Criteria

- [ ] https://wingdrive.org/privacy and /service-terms are live in both languages.
- [ ] The domain shows as verified in Google Cloud Console.
