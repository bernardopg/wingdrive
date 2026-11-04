---
id: DESK-049
title: Folders cannot be tagged from the UI
status: Done
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
sprint: S04
tags: [desktop, tags, explorer, bug]
last_updated: 2026-11-03
---

## Description

Tag mode (Alt+G) and the Inspector apply tags only to items with a content identity. Folders never have one,
so a folder, even an indexed one, cannot be tagged from the UI: tag mode shows "These files need to be indexed
before they can be tagged" and the Inspector shows its "tags after indexing" note. The core already accepts
`EntryUuid` targets (`wing tag apply` tags a folder by entry id), and tagged folders display their tag.

Found during the INDEX-010 desktop check on 2026-11-03.

## Implementation Steps

1. Tag mode: fall back to `EntryUuid` targets for selected items without a content identity but with a
   persistent entry, as the Inspector's add button already does.
2. Inspector: treat an item as taggable when it has a persistent entry, not only a content identity.
3. Keep the message for items that are only ephemeral.

## Acceptance Criteria

- [x] An indexed folder can be tagged and untagged from tag mode and from the Inspector
- [x] Ephemeral-only items still explain why they cannot be tagged
- [x] Verified on the Linux desktop build

## Evidence

- Tag mode tags items without a content identity as `EntryUuid` targets (files keep `Content`), and a second
  press on a tag every selected item has removes it (`tags.unapply`). Errors that mean "not indexed" show the
  index-first message.
- Inspector: folders show the tag editor; a failed add shows "Tags are available after indexing this location".
- Desktop debug build, 2026-11-03 (Xvfb, private D-Bus, isolated daemon, WebKitWebDriver), library Test with
  location `loc`: tag mode on `docs` added the tag (dot in the grid, row in `user_metadata_tag`) and a second
  `1` removed it; the Inspector's Add tags and × did the same; tag mode on `readme.txt` tagged its content.
  On `eph`, a folder outside the location, tag mode and the Inspector both showed the index-first message.
