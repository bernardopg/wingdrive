---
id: MOB-004
title: "Native core logs and JNI error handling"
status: Done
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S03
tags: [mobile, rust, android, logging]
last_updated: 2026-10-17
---

## Description

`modules/wing-mobile-core/core/src/lib.rs` reports init failures with `println!`, which goes to
`/dev/null` on Android, and prints every RPC request and response. The JNI entry points `unwrap()`
string and JVM calls, so a bad call aborts the process instead of rejecting the promise.
`attach_current_thread()` attaches and detaches the JVM thread on every response.

## Acceptance Criteria

- [x] Every `println!` replaced by `tracing` at the right level; RPC bodies no longer logged
- [x] Init failures visible in logcat
- [x] JNI entry points reject the promise instead of panicking on string or JVM errors
- [x] Callback threads attach once with `attach_current_thread_permanently`
- [x] `cargo clippy -p wing-mobile-core --target aarch64-linux-android` clean

## Evidence

- `lib.rs`: no `println!` left; RPC request/response and per-event dumps removed; `init_tracing`
  writes to logcat (`WingDriveCore`) through `LogcatWriter` on Android.
- JNI: string, NUL and global-ref failures reject the promise or log instead of panicking;
  `call_with_string` uses `attach_current_thread_permanently`; event and log registration share
  `register_listener`; `handle_core_msg` checks both `RUNTIME` and `CORE`.
- Verified 2026-10-17 on the emulator: core init, migrations and volume detection lines in logcat.
- `cargo ndk -t x86_64 clippy -p wing-mobile-core`: no warnings in the crate.
