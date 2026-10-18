---
name: wingdrive-mobile-dev
description: Fast develop, run and debug loop for the WingDrive mobile app (apps/mobile, Expo + embedded Rust core) on the Android emulator, a USB Android device, or iOS. Use when building, installing, running, testing, screenshotting or debugging the mobile app, when an APK rebuild feels slow, when the app crashes on launch, or when reading logcat for WingDriveCore panics.
---

# WingDrive mobile dev loop

The mobile app (`apps/mobile`) embeds the Rust core through JNI (`modules/wing-mobile-core`).
A release APK rebuilds Rust, runs R8 and bundles JS, which takes minutes. Never use
`assembleRelease` to iterate. Install the debug dev client once and reload JS from Metro.

Full reference: `docs/core/mobile-development.mdx`.

## Decide what to rebuild

- Changed `apps/mobile/src` or `packages/ts-client`: nothing to build. Metro Fast Refresh reloads it.
- Changed Rust (`modules/wing-mobile-core/core`, `core/`), Kotlin, `android/`, or a package with
  native code: run `just mobile-install-android [abi]`, then relaunch the app.
- Need real performance, LAN pairing or the camera: use a USB device, and a release build only for
  the final check.

## Android loop

Run from the repo root, in this order:

1. `adb devices`. If no emulator runs, start one in the background: `just mobile-emulator`
   (AVD `wingdrive-api36`, x86_64). Wait for `adb shell getprop sys.boot_completed` to print `1`.
2. If the dev client is not installed or native code changed: `just mobile-install-android`
   (emulator, x86_64) or `just mobile-install-android arm64-v8a` (USB device). The first Rust
   build for a new ABI takes long; later builds are incremental.
   With several devices attached, set `ANDROID_SERIAL` (from `adb devices`) so Gradle installs on one.
3. Start Metro in the background: `just dev-mobile`. For a USB device also run
   `adb reverse tcp:8081 tcp:8081`.
4. Launch: `adb shell monkey -p com.wingdrive.app -c android.intent.category.LAUNCHER 1`.

The ABI passed to Gradle must match the `.so` that `build.sh` produced (`WING_ANDROID_ABIS`).
A mismatch crashes at launch with `UnsatisfiedLinkError`.

## Debug

- JS errors and `console.log`: the Metro terminal output.
- Native and crash logs: `adb logcat -d -s WingDriveCore WingMobileCore ReactNativeJS AndroidRuntime`.
  Clear first with `adb logcat -c` to see only the current run.
- Rust panics at the JNI boundary log under the `WingDriveCore` tag.
- `FATAL EXCEPTION ... JavascriptException` is a JS error. Read the `ReactNativeJS` lines above it.
- If a `mobile-mcp` MCP server is available, use it to screenshot, read the accessibility tree,
  tap and type. Otherwise use `adb exec-out screencap -p > shot.png` and `adb shell input tap X Y`.

## Multi-device testing

1. Start a disposable desktop daemon: `wing --data-dir /tmp/wd-desk --instance mobilepair start`.
2. On the phone or emulator open the Network tab, tap Generate Code, then Generate Pairing Code.
3. Read the code: `adb exec-out uiautomator dump /dev/tty` shows the words; for the emulator (NAT)
   decode the QR with `adb exec-out screencap -p > s.png && zbarimg --raw s.png` to get `node_id`.
4. `wing --data-dir /tmp/wd-desk --instance mobilepair network pair join "<words>" [--node-id <id>]`.
5. Check `wing ... network devices` and the phone's Network tab.

Never inject text with `adb shell input text` into a field inside a React Native `Modal`: the keys
reach the activity and can open the dev menu or system settings. Never toggle system settings.

## Gotchas

- Import navigation from `expo-router`, never `@react-navigation/*` (Expo Router 57 vendors it).
- Duplicate copies of `@tanstack/react-query`, `react` or `react-native` in the Bun tree break
  context at runtime ("No QueryClient set"). Check `bunx expo-doctor` duplicates.
- `apps/mobile/scripts/patch-svg.sh` uses macOS `sed -i ''`; it is not needed for Android.
- Use NDK `27.1.12297006` (`ANDROID_NDK_HOME`), the version React Native 0.86 expects.
- The emulator is behind NAT (`10.0.2.x`): LAN discovery and desktop pairing do not work there.

## iOS

Xcode and the Simulator run only on macOS. Build the dev client on a Mac with
`just dev-mobile-ios`; it can load JS from Metro on any machine. On Linux, say that iOS cannot be
built locally rather than trying.
