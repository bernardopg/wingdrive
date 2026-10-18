---
id: NET-005
title: Wingdrop between paired devices
status: Done
assignee: bernardopg
parent: NET-000
priority: High
tags: [networking, wingdrop, sharing, p2p]
milestone: M2
sprint: S03
last_updated: 2026-10-18
---

## Description

Send files and folders to a paired device the way AirDrop does: the receiver sees an offer, accepts or
declines it, and accepted files land in its Wingdrop folder. Replaces `network.spacedrop.send`, which
only returned an error. Unpaired nearby devices stay in NET-003.

## Implementation Steps

- [x] Offer, decision, finished and cancelled messages over the authenticated messaging channel
- [x] Receiver reserves a free name per item (nothing is overwritten) and grants the sender write access
      to exactly those paths until the drop ends; chunks of a cancelled drop are refused
- [x] Sending job: offer, wait up to five minutes, push every file with end-to-end hashes, recreate
      empty folders, stop at once when either side cancels
- [x] Settings: receive on/off, accept without asking, destination folder (Downloads by default)
- [x] CLI: `wing network wingdrop send|pending|accept|decline|cancel`
- [x] UI: "Send with Wingdrop" in the file menu, an offer prompt, a received toast with
      "Show in folder", and the Wingdrop window (View menu) listing the real paired devices

## Acceptance Criteria

- [x] Unit tests: authenticated sender, unsafe names, free-name reservation, grants, cancel cleanup,
      expiry, decisions routed only to their drop, planning of folders and empty folders
- [x] Two daemons on the same host: folder and file delivered byte for byte, a second drop got
      `nota 2.txt`, decline, auto-accept, receiver cancel stops the sender at 7%, sender cancel
      removes the partial file on the receiver
- [x] Tauri app: send from the file menu, accept an offer from the prompt

## Notes

- Requests on the messaging channel are retried once on a fresh connection when the cached one closes
  mid-request; reconnection could close it under a request.
- Settings changed while the daemon ran were reverted at shutdown; fixed with this task.
