---
id: CORE-027
title: Upgrade Wasmer past 7.1.0
status: To Do
assignee: bernardopg
parent: PLUG-000
priority: Low
milestone: M5
sprint: S06
tags: [core, plugins, wasm, dependencies, windows]
related_tasks: [PLUG-001]
last_updated: 2026-11-28
---

## Description

`core/Cargo.toml` pins `wasmer = "=7.1.0"`. Dependabot's cargo group of 2026-11-28 (#578) moved it
to 7.5.0 and CI failed on two counts:

- Linux: `future cannot be sent between threads safely` in `core/src/infra/daemon/rpc.rs`
  (`tokio::spawn` over `Arc<Core>`). Since 7.4 Wasmer's store holds a
  `dyn FnOnce(StoreMut<'a>) -> Result<OnCalledAction, ...>` callback that is not `Send`.
- Windows: `wasmer-compiler-cranelift` 7.5.0 stops with "The Cranelift compiler backend is not
  supported on Windows. Use the V8 backend instead."

Wasmer is ignored in `.github/dependabot.yml` until this task is done.

## Implementation Steps

- [ ] Check whether a newer Wasmer restores `Send` on the store, or keep the store off the
      `Send` path (own thread or `spawn_local` for the extension runtime).
- [ ] Pick a backend that builds on Linux, Windows and macOS (Cranelift, Singlepass, LLVM or V8).
- [ ] Bump the pin, drop the Dependabot ignore and the comment above the pin.

## Acceptance Criteria

- [ ] `cargo clippy --workspace` passes on Linux and Windows CI with the new Wasmer.
- [ ] The test extension loads and runs (`extensions/test-extension`).
