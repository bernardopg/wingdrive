---
id: DESK-049
title: Folders cannot be tagged from the UI
status: To Do
assignee: bernardopg
parent: DESK-000
priority: Medium
milestone: M2
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

- [ ] An indexed folder can be tagged and untagged from tag mode and from the Inspector
- [ ] Ephemeral-only items still explain why they cannot be tagged
- [ ] Verified on the Linux desktop build
