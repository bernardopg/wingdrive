---
id: ADAPT-013
title: "Thunderbird local mail adapter"
status: To Do
assignee: bernardopg
parent: ADAPT-000
priority: High
milestone: M5
sprint: S07
tags: [adapters, mail, thunderbird]
depends_on: [ADAPT-004, ADAPT-008]
related_tasks: [ADAPT-020]
last_updated: 2026-11-24
---

## Description

Index mail stored by Thunderbird without any OAuth: local folders and the IMAP cache (mbox and
maildir), with sender, recipients, subject, date, body text and attachment names. Read only; never
touch Thunderbird's files while it writes them. Real data: `~/.thunderbird` (2.1 GB).

## Implementation Steps

- [ ] Profile detection from `profiles.ini` on every platform
- [ ] mbox and maildir readers; `.msf` summaries ignored; decoding of MIME, encodings and HTML bodies to text
- [ ] Thread relations from `Message-ID`, `In-Reply-To` and `References`
- [ ] Incremental sync by mailbox offset and size; messages removed from a folder removed from the source

## Acceptance Criteria

- [ ] Lives in its own repository `<org>/wingdrive-adapter-<id>` created from the template (ADAPT-008), with README, license, icon and tests
- [ ] Listed in the official registry through a reviewed pull request that the registry bots (ADAPT-007) pass
- [ ] Installs from the store in the desktop app and from `wing adapters install <id>`
- [ ] Defaults: the source path or account is detected on Linux and macOS (and Windows once ADAPT-002 lands); a wrong value fails `sources.create` with a message that names the field
- [ ] Declares only the permissions it uses (ADAPT-009); runs with network off unless it declares hosts
- [ ] Sync is incremental: a second run with no changes writes nothing new, a changed item is updated in place, and an item removed upstream is removed from the source
- [ ] Items show in Sources > source > items with title, subtitle, date and preview, and are found by the library search
- [ ] Script tests with fixtures cover parsing, cursors, deletions and malformed input, and the contract harness (ADAPT-003) passes in the adapter's CI
- [ ] Real proof on the author's machine: install from the store, sync of real data with item counts, second run idempotent, items opened in the desktop app (screenshot in the PR)
- [ ] Store listing on wingdrive.org/adapters and a user page (EN and PT) with setup, what is indexed and privacy notes
