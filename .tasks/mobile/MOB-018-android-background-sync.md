---
id: MOB-018
title: "Android background sync: keep the core connected while the app is closed"
status: In Progress
assignee: bernardopg
parent: MOB-000
priority: High
milestone: M5
sprint: S07
tags: [mobile, android, sync, networking]
related_tasks: [MOB-000, MOB-012, MOB-017]
last_updated: 2026-11-28
---

## Description

The author noticed that the Android app stops syncing with paired devices, receiving files and
indexing as soon as it leaves the screen. MOB-000 lists "Background sync" as open work. This task
records why and adds a foreground service that keeps the core running.

## Findings (2026-11-28)

What the app has today:

- The core runs inside the app process: `initialize_core` creates a Tokio runtime and the `Core`
  in Rust statics, the JavaScript client starts it at launch, and `shutdown_core` does nothing.
  Nothing ties it to the screen, so it keeps running while Android lets the process run.
- The app has no Android `Service`, no WorkManager job, no `expo-task-manager`,
  `expo-background-fetch` or notifications package, and no wake lock. The manifest declares no
  foreground service permission.
- Android therefore treats the backgrounded app as a cached process. With the screen off, Doze
  blocks its network, and stock Android 14+ also freezes cached processes and kills them under
  memory pressure.

Measured on the moto g(7) with the 3.8.2 release (LineageOS 22.2, Android 15, cached app freezer
off on this ROM): the app was in the background with the screen off and the device forced into
Doze (`dumpsys deviceidle force-idle`). A file added to the laptop's `~/WingDrive` location did not
reach the phone; the core logged 281 `error sending mDNS: Operation not permitted (os error 1)`
lines in 90 seconds (the network policy rejects its sockets), and the laptop listed the phone as
`Paired` instead of `Connected`. The author's belief is right.

## Design

- `WingCoreService`, a foreground service of type `connectedDevice` with a low-importance
  persistent notification. Foreground priority keeps network access and honours wake locks in
  Doze; a partial wake lock keeps the core's timers and keepalives running with the screen off,
  and a multicast lock keeps local discovery working. `connectedDevice` fits a service that talks
  to the user's own devices, has no daily time limit (unlike `dataSync` on Android 15), and its
  Android 14 prerequisite is met by `CHANGE_WIFI_MULTICAST_STATE`, which the app already holds.
- Settings gains a Background section (Android only) with "Keep running in the background",
  off by default. Turning it on asks for the notification permission on Android 13+ and starts
  the service. While it is on, Settings offers "Allow unrestricted battery use" (the system's
  battery optimization exemption request) and "Allow notifications" when those are missing.
- The choice is stored natively, so the app starts the service on every launch while it is on.
  When Android restarts the service after killing the process (`START_STICKY`), no JavaScript
  runs, so the service starts the core itself with the data folder and device name the app last
  used. `initializeCore` became a static native method for that; the JNI symbol is unchanged.

## Implementation Steps

- [x] Investigate what keeps or stops the core in the background and measure it on the moto g(7)
- [x] `WingCoreService` with notification, wake lock, multicast lock and core start
- [x] Manifest: service, `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_CONNECTED_DEVICE`,
      `POST_NOTIFICATIONS`, `WAKE_LOCK`, `REQUEST_IGNORE_BATTERY_OPTIMIZATIONS`
- [x] Native module: `setBackgroundSync`, `getBackgroundSync`, `openBatteryOptimizationSettings`,
      service start on launch
- [x] Settings section with en and pt strings, unit tests for the hint logic
- [ ] Verify on the moto g(7): app in the background, screen off, Doze, file added on the laptop
      reaches the phone

## Acceptance Criteria

- [ ] With the toggle on, the app in the background and the screen off (Doze forced), a file added
      to a paired desktop's shared location reaches the phone without opening the app.
- [ ] The notification shows while the service runs and disappears when the toggle is turned off.
- [ ] With the toggle off, nothing changes from 3.8.2: no service, no notification.

## Out of scope

- Starting at boot. The service needs the app opened once after a restart; Android 15 allows
  `connectedDevice` services from `BOOT_COMPLETED`, so this can follow.
- iOS, which has no equivalent of a long-running foreground service.
