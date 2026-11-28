---
id: INTG-003
title: Google OAuth app verification (sensitive and restricted scopes)
status: Done
assignee: bernardopg
parent: INTG-000
priority: High
milestone: M5
sprint: S05
tags: [integrations, google, oauth, verification, security]
last_updated: 2026-11-28
related_tasks: [INTG-001, INTG-002, INTG-011]
---

## Description

While the OAuth app is in "Testing", only listed test users can sign in and their grants expire after 7 days. Moving to "In production" without verification shows the "Google hasn't verified this app" screen and caps the app at 100 users. Full Drive access (`https://www.googleapis.com/auth/drive`) is a restricted scope: verification for it needs Google's brand review plus a yearly third-party security assessment (CASA). The Photos scopes are sensitive and need brand and scope review.

Decide the scope strategy before submitting, because it changes the product:

- Keep `drive` (browse all of My Drive as a volume, which is what CLOUD-004 ships) and go through CASA every year.
- Or switch to `drive.file` (not restricted): WingDrive only sees files it created or that the user opens through the Google Picker. Browsing the whole Drive and syncing existing folders would stop working.

## Implementation Steps

1. Record the scope decision as an ADR in `docs/design/`.
2. Prepare the verification package: scope justifications, a demo video of each scope in use, links from INTG-002.
3. Submit brand verification, then sensitive and restricted scope verification.
4. If `drive` stays, run the CASA assessment (tier per Google's request) and schedule the yearly renewal.
5. Publish the app to "In production" once approved.

## Progress (2026-11-13)

The app is "In production" without verification: grants persist, the unverified-app screen shows
and the project is capped at 100 users. Brand verification started with the domain check; a
120x120 logo is in `docs/logo/wingdrive-logo-120.png` for the consent screen. The scope decision
(`drive` with CASA or `drive.file`) is still open.

## Decision (2026-11-16)

No recurring cost for now: the bundled client asks for `drive.file` and the custom-client mode keeps
full `drive` (ADR in `docs/design/GOOGLE-DRIVE-SCOPE.md`). The Data Access page lists only
non-restricted scopes; the two sensitive Photos scopes carry the justification. Branding and the
Search Console domain property for `wingdrive.org` are verified (TXT record added through the
Hostinger API). Demo video: https://youtu.be/qsLpemmndok (unlisted, personal photos and other accounts hidden). Verification request sent the same day.

## Progress (2026-11-17)

Google sent the automated checklist ("[Action Needed] Your Google APIs Verification Request") and
asked for a reply. Audit result: the consent screen, the demo video and the privacy policy already
covered the sensitive scopes. Changes for least privilege and completeness: the unused
`photoslibrary.edit.appcreateddata` scope was dropped (INTG-011), the Drive Picker extends
`drive.file`, and the privacy policy now states what is cached locally, for how long, and that
traffic uses HTTPS. After the release ships, update Data Access (remove the scope, new video) and
reply to the email.

## Progress (2026-11-18)

v3.5.0 shipped the Picker and the smaller scope set. Data Access now lists exactly the app's scopes
(`photoslibrary.edit.appcreateddata` removed), with an updated justification and a new unlisted demo
video, https://youtu.be/YghrdeRQiLM (consent screen in English with every scope, the client ID in
the address bar, each scope's effect in Drive and Photos). The checklist email was answered on
2026-11-18 with the audit per item. Data Access is back "em análise"; wait for Google.

> 2026-11-22 (DEV-008): moved to `To Do` during S05 board cleanup. Blocked on third party: waiting
> for Google's OAuth verification review response. No active internal work in this sprint.

## Progress (2026-11-28)

Google Auth Platform > Verification Center reports both parts verified: "Your branding has been
verified and is shown to users" and "Your app's data access has been verified". Audience: External,
"In production". Data Access lists `openid`, `userinfo.email`, `userinfo.profile`, `drive.file` and
`photoslibrary.readonly.appcreateddata` as non-sensitive, `photoslibrary.appendonly` and
`photospicker.mediaitems.readonly` as sensitive, and no restricted scope. Signing in with the
packaged 3.8.2 build showed WingDrive's name and links, with no "Google hasn't verified this app"
screen (INTG-004). The 100-user cap counter still shows on the Audience page; Google says it is not
enforced for verified scopes.

## Logo submission (2026-11-28)

The author decided to add the logo even though it triggers a new brand review. Steps on
Google Auth Platform > Branding: uploaded `docs/logo/wingdrive-logo-120.png` (PNG, 120x120,
4.9 KB), saved, then "Verificar marca". No form asked for scope justifications, a demo video or
other data: the review was the automated brand check, which passed within five minutes. The
Branding page then offered "Publicar branding" (the verified result expires after 7 days if not
published); it was published right away. Status shown afterwards: Branding "Sua marca foi
verificada e está aparecendo para os usuários" with the new logo; Verification Center reports
branding verified and "O acesso aos dados do seu app foi verificado". Name, support email,
homepage, privacy and terms links, authorized domain and developer contacts were left as they
were; scopes, clients, secrets and API keys were not touched. The task stays Done.

## Acceptance Criteria

- [x] Any Google account signs in without the unverified-app warning (Verification Center reports branding and data access verified; checked with the author's account on 2026-11-28).
- [x] Grants no longer expire after 7 days (published "In production").
- [x] No restricted scope is requested by WingDrive's own client, so no yearly CASA assessment is needed (decision of 2026-11-16).
