---
title: Devices and sync
description: Pair your own devices, sync the library between them and send files with Wingdrop.
order: 6
group: Using WingDrive
---

WingDrive connects devices you own directly to each other. There is no WingDrive server in the middle: connections run over Iroh/QUIC, end-to-end encrypted.

## Pair two devices

Pairing uses a one-time code. One device **generates** it, the other **joins** with it.

- **12 words** (a BIP39 mnemonic): easy to read and type, works on the same local network.
- **QR code**: carries the same words plus the device's node ID, so it works across networks and the internet, with relay fallback.

Codes are valid for five minutes and never reused.

From the desktop app, open **Devices** in the sidebar and choose *Pair device*. From the CLI:

```bash
# On the first device
wing network pair generate

# On the second device
wing network pair join "brave lion sunset river eagle mountain forest ocean thunder crystal diamond phoenix"

# Follow progress
wing network pair status
```

### Security

- **Authentication**: devices prove their identity with Ed25519 signatures.
- **Confidentiality**: all traffic is encrypted with session keys.
- **Integrity**: a challenge-response proves the joiner knows the code.
- **Forward secrecy**: new keys for every session.

Paired devices send their current name, type, operating system and app version every time they connect, so a renamed or reinstalled device shows up with its new description.

## Remove a pairing

```bash
wing network devices
wing network revoke <device>
```

Revoking tells the other device, which drops the pairing too. A device that is offline keeps the pairing until you remove it there.

## Library sync

Library sync shares the **index**, tags and metadata between paired devices, so every device can browse and search what the others have. File contents stay where they are.

```bash
wing library sync-setup setup --action share-local   # on the device that has the library
wing library sync-setup setup --action join-remote   # on the device that joins
```

The setup offers three actions: `share-local` (share this library), `join-remote` (join a library from the other device) and `merge`.

## Wingdrop

Wingdrop sends files to a paired device:

```bash
wing network wingdrop send <device-id> ./photo.jpg
```

File sync between devices, public sharing links and Wingdrop to nearby unpaired devices are planned for milestone M4.
