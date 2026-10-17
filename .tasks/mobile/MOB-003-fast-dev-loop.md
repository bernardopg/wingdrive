---
id: MOB-003
title: "Fast dev loop: debug dev client, emulator and agent tooling"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, development, android, tooling]
last_updated: 2026-10-17
---

## Description

Each change was tested by building a release APK and installing it on the Moto G7, which takes
minutes. Use the debug dev client with Metro, an x86_64 emulator, and an MCP server so agents can
drive the device.

## Acceptance Criteria

- [x] AVD `wingdrive-api36` (android-36 Play Store x86_64) created; scrcpy, sccache, watchman and the
      `x86_64-linux-android` target installed
- [x] `build.sh` builds the ABIs in `WING_ANDROID_ABIS`, uses sccache and deletes hashed `.so` copies
- [x] `just mobile-emulator` and `just mobile-install-android [abi]`
- [x] `docs/core/mobile-development.mdx`, skill `.agents/skills/wingdrive-mobile-dev`, AGENTS.md entry
- [x] `mobile-mcp` configured in Claude Code, Codex and OpenCode
- [x] Debug dev client installed on the emulator and a JS edit reloads without a rebuild

## Evidence

- First `just mobile-install-android` (x86_64): BUILD SUCCESSFUL in 11m 29s, installed on
  `emulator-5554`; Metro bundled 5851 modules in 28.6 s and the dev client loaded it.
- Overview scroll fix on the Moto G7 applied through Fast Refresh in about 1 s, no rebuild. Incremental arm64 rebuild: 14m 53s, mostly Rust release.
