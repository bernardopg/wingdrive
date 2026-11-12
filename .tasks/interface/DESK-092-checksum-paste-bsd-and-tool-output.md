---
id: DESK-092
title: Pasting a BSD-style or openssl checksum line in the Inspector says it is not a hash
status: Done
assignee: bernardopg
parent: DESK-000
priority: Low
milestone: M2
sprint: S05
tags: [desktop, inspector, bug]
last_updated: 2026-11-11
---

## Description

The SHA-256 compare field of the Inspector (DESK-020) accepts a bare digest, a `sha256:` prefix and a
`sha256sum` line, because `parseExpectedSha256` only looks for the digest at the start of the text.
Many projects publish the BSD tag format instead (`sha256sum --tag`, `shasum -a 256 --tag`, FreeBSD
`CHECKSUM.SHA256` files), and people paste `openssl dgst -sha256` or Windows `certutil -hashfile`
output. Each of these shows "invalid" for a digest that matches. Found while checking DESK-020 on
2026-11-11.

## Steps to reproduce

1. Run `sha256sum --tag abc.txt`: `SHA256 (abc.txt) = ba7816bf...15ad`.
2. Select `abc.txt`, click "Calculate SHA-256" in the Inspector and paste that line.
3. The field reports the text holds no digest instead of a match.

## Acceptance Criteria

- [x] BSD tag, `openssl dgst` and `certutil` output compare against the digest
- [x] Text with two different digests stays ambiguous and is not compared
- [x] `checksum.test.ts` covers these forms

## Fix

`parseExpectedSha256` now takes the one 64-character hex token in the pasted text, wherever it sits,
instead of only a digest at the start. A token must stand alone, so a 65-character run or a longer
word never counts, and text with two different digests returns null, so the field says it holds no
usable hash rather than comparing against a guess. Two new cases in `checksum.test.ts` cover the BSD
tag line, `openssl dgst`, a `sha256sum -b` line, `certutil` output with CRLF line ends, a repeated
digest and two different digests; `bun test
packages/interface/src/components/Inspector/variants/checksum.test.ts` passes 5 tests.
