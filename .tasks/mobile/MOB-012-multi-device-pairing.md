---
id: MOB-012
title: "Multi-device pairing: real Network screen and device identity"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, network, pairing, core]
last_updated: 2026-10-17
---

## Description

Testing desktop and two Android devices together on 2026-10-17 exposed four gaps. The Network tab
was hidden and its screen showed hardcoded status, devices and sync. Every phone registered as
"WingDrive Mobile", so two phones were indistinguishable. Every peer reported `DeviceType::Desktop`
because `get_local_device_info` ignored the detected form factor. The mobile join field asks for
hyphen-separated words, but `PairingCode::from_string` only split on whitespace.

## Acceptance Criteria

- [x] Network tab visible; screen shows `network.status`, `network.devices.list` (polled every 5 s),
      pair buttons, and unpair on long press (`network.device.revoke`)
- [x] New Android installs are named after the phone (`Platform.constants` brand and model)
- [x] Peer device type comes from `DeviceConfig.form_factor`
- [x] Pairing codes accept spaces or hyphens; unit test `from_string_accepts_spaces_and_hyphens`
- [x] Library list re-renders on switch (`useCurrentLibraryId`); switcher opens from `⋯` and shows a short id
- [x] Overview scrolls on Android (`pointerEvents="box-none"` removed from the `Animated.ScrollView`)
- [x] Desktop pairs with the Moto G7 over the LAN (mDNS) and with the emulator through the relay
- [x] Mobile peers report type `Mobile` after the native rebuild
- [x] Library sync between desktop and a phone (`network.sync_setup`) verified

## Evidence

- 2026-10-17: isolated desktop daemon (`wing --instance mobilepair`). Moto G7 generated words,
  `wing network pair join "<words>"` paired over the LAN; the phone's Network tab listed
  `archlinux · Desktop · linux · Connected`, 1/1.
- Emulator (NAT `10.0.2.x`) generated a QR; `zbarimg` decoded it from a screenshot and
  `wing network pair join "<words>" --node-id <id>` paired through the relay as
  `google sdk_gphone64_x86_64`.
- After the native rebuild and a re-pair through the relay, the desktop lists the emulator as
  `Type: Mobile`. Already-paired peers keep the info sent at pairing time (follow-up below).
- Library sync: `wing library sync-setup setup --action share-local` shared the desktop library to
  the Moto G7. The phone's `library.db` matched the desktop (5 entries, 1 location, 2 devices); the
  phone browsed `shared/docs` and listed `nota1..3`. A file created on the desktop appeared on the
  phone's open listing about 2 s later without a refresh.
- Unpair from the phone's Network tab (long press) works.

## Follow-ups

- MOB-013: peer device info is only exchanged at pairing; renames and type fixes never reach paired
  devices. `network revoke` on one side leaves the other side paired and "Connected".
- MOB-014: `wing library sync-setup setup` help lists `register-only, create-shared` but the parser
  accepts `share-local, join-remote, merge`; without `--local-device` it fails with "Device config
  not found" under `--data-dir`/`--instance`.
- The phone has no locations of its own yet (MOB-008), so phone-to-desktop sync is untested.
