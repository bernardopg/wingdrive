---
id: TAURI-014
title: Linux desktop runtime robustness and search parity
status: To Do
assignee: bernardopg
parent: TAURI-000
priority: High
milestone: M1
sprint: S02
tags: [tauri, linux, interface, regression]
last_updated: 2026-10-05
---

## Description

Second half of the desktop UI audit, split from TAURI-013. Startup failures and window restarts must recover on their own. Auxiliary windows need the same providers as the main window. Search must match the explorer views. The search and recents checks still open in TAURI-006 are validated here.

## Implementation Steps

- [ ] Bound startup failure and recover isolated per-window event subscriptions
- [ ] Compose feature providers, dialogs, notifications and reconnect handling in auxiliary windows
- [ ] Complete search scope/filter/sort/pagination/debounce and view parity
- [ ] Preserve streamed text and improve clipboard, drag, Jobs, source retry and pairing
- [ ] Add targeted tests and Linux validation in the packaged bundle

## Acceptance Criteria

- [ ] Startup failures are recoverable; restart and window cleanup preserve scoped subscriptions
- [ ] Jobs and Settings auxiliary windows have their required contexts and feedback
- [ ] Search and preview regressions are covered by tests; pagination and source retries are explicit
- [ ] Search and recents click-through pass in the packaged Linux app (closes the TAURI-006 gap)
- [ ] Frontend tests, typecheck/build and relevant Rust validation pass, with limitations recorded

## Validation

Related: 10ab25d (auxiliary windows that never reached app_ready stayed hidden) already landed on main.
