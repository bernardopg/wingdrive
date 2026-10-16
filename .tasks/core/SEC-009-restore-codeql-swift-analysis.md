---
id: SEC-009
title: "Restore CodeQL Swift analysis for the iOS app"
status: To Do
assignee: bernardopg
parent: SEC-008
priority: Medium
milestone: M5
tags: ["ci", "security", "codeql", "mobile"]
last_updated: 2026-10-16
---

## Description

PR #170 moved CodeQL to an advanced setup with the security-extended suite, but the
Swift language had to be left out of the matrix. The iOS build that CodeQL traces takes
57 minutes on its own, so the job hit its own 60-minute timeout and was cancelled on
every run.

The other four languages (`actions`, `javascript-typescript`, `python`, `rust`) analyze
on every pull request with the `security-extended` suite. Until this task lands, Swift
and Objective-C code under `apps/mobile/ios` and `packages/swift-client` has no CodeQL
coverage.

## Implementation Steps
- [ ] Measure the real `xcodebuild -workspace WingDrive.xcworkspace -scheme WingDrive -sdk iphonesimulator` time on a clean macOS runner
- [ ] Cache CocoaPods (`apps/mobile/ios/Pods`) so `pod install` stops fetching every run
- [ ] Decide between a larger per-language timeout, a self-hosted runner, or splitting the analysis from the build
- [ ] Re-add the `swift` matrix entry plus its `Setup Bun`, `Install mobile dependencies` and `Install CocoaPods dependencies` steps in `.github/workflows/codeql.yml`
- [ ] Confirm `.github/codeql/codeql-config.yml` needs no path change for Swift source

## Acceptance Criteria
- The `Analyze (swift)` job finishes inside its timeout on pull requests
- Alerts raised on Swift source appear in the Code Scanning tab with the `/language:swift` category
- The remaining four CodeQL jobs keep passing

## WingDrive triage (2026-10-16)

M1 closed without it. Swift analysis matters only for the iOS build of the mobile app, so it moves
with mobile to M5.
