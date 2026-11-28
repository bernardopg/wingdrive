---
id: INTG-002
title: Privacy policy, terms and Limited Use disclosure on wingdrive.org
status: Done
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, website, legal]
last_updated: 2026-11-28
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

> 2026-11-22 (DEV-008): moved to `To Do` during S05 board cleanup. Waiting on third-party/domain
> verification (Search Console DNS propagation) and the remaining About-page link; not actively in
> progress.

`website/` has `/privacy` and `/service-terms` in English and Portuguese (`LegalPage.astro`,
strings in `src/i18n/strings.ts`), linked from the footer. The privacy policy lists the Google and
Microsoft data WingDrive accesses, how it is used and stored, how to revoke access, and the Limited
Use sentence. The author added the Search Console TXT record for `wingdrive.org` at Hostinger;
DNS propagation pending. A link from Settings > About is still to do.

2026-11-28: the four pages answer 200 (`/privacy/`, `/service-terms/`, `/pt/privacy/`,
`/pt/service-terms/`) and the English policy carries the Limited Use sentence. The
`google-site-verification` TXT record resolves for `wingdrive.org`. Settings > About links the
privacy policy and terms in the UI language (`settingsPages.test.tsx`). Left: the author confirms
the domain shows as verified in Google Cloud Console.

2026-11-28, checked in the browser with the author's account: the Search Console domain property
`sc-domain:wingdrive.org` shows "verified owner" (added 2026-11-13). Cloud Console no longer has a
separate domain verification page; Google Auth Platform > Branding lists `wingdrive.org` as the
authorized domain and the Verification Center says "Your branding has been verified and is shown
to users". The consent screen links https://wingdrive.org/privacy and
https://wingdrive.org/service-terms.

## Acceptance Criteria

- [x] https://wingdrive.org/privacy and /service-terms are live in both languages (checked 2026-11-28, all four pages 200).
- [x] The domain shows as verified in Google Cloud Console (checked 2026-11-28: Search Console lists the author as verified owner of the `wingdrive.org` domain property, and Google Auth Platform > Verification Center reports the branding as verified).
