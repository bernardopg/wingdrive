# Coverage

[![Coverage](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcoverage.json)](https://bernardopg.github.io/wingdrive/)
[![Rust](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Frust.json)](https://bernardopg.github.io/wingdrive/)
[![Frontend](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Ffrontend-coverage.json)](https://bernardopg.github.io/wingdrive/)

Line coverage of the tests WingDrive runs in CI. The [Coverage workflow](https://github.com/bernardopg/wingdrive/actions/workflows/coverage.yml)
measures it on every push to `main` and every Monday, so the numbers on this page always show the
latest run and nobody edits them by hand.

**[Open the full report](https://bernardopg.github.io/wingdrive/)** for every file, filterable and sortable.

## By component

| Component | Coverage | What it covers |
|---|---|---|
| Core | ![core](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-core.json) | Daemon, operations, indexing, sync (`core/`) |
| Desktop | ![desktop](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-desktop.json) | Tauri shell, IPC, terminal, launch handling and its crates (`apps/tauri/`) |
| CLI | ![cli](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-cli.json) | `wing` (`apps/cli/`) |
| Server | ![server](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-server.json) | `wing-server` (`apps/server/`) |
| Shared crates | ![crates](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-crates.json) | Crypto, archive, file watcher, branding and the rest of `crates/` |
| Mobile core | ![mobile](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-mobile.json) | Rust core embedded by the mobile app (`apps/mobile/`) |
| Interface | ![interface](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-interface.json) | React UI shared by desktop, web and mobile (`packages/interface/`) |
| TypeScript client | ![ts-client](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-ts-client.json) | Generated types, hooks and transport (`packages/ts-client/`) |
| Other packages | ![packages](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fbernardopg%2Fwingdrive%2Fbadges%2Fcomponent-packages.json) | Primitives, AI and assets (`packages/`) |

"no tests" means no test touches that component yet.

## How it is measured

- **Rust:** [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) runs
  `scripts/coverage-rust.sh`: the tests from `scripts/ci-rust.sh tests` (the same ones the CI gate
  runs, including the CLI contracts) plus the unit and integration tests of the server and shared crates. Doctests
  are not run.
- **Frontend:** `scripts/coverage-frontend.sh` runs every `bun test` file from the CI workflow with
  `--coverage`, one process per file, as CI does.
- **Report:** `scripts/coverage-report.py` merges the lcov files. A line counts as covered when any
  test ran it. Test files, benchmarks, generated code and dependencies are left out. It writes the
  badges (branch [`badges`](https://github.com/bernardopg/wingdrive/tree/badges)), the HTML report
  (GitHub Pages) and the table on each workflow run page.

The percentages count instrumented lines: Rust code that no test binary compiles and TypeScript
files that no test imports do not appear in the totals. The report lists the files that do.

The [COV-00 plan](.tasks/testing/README.md) defines progressive targets for the core, desktop,
CLI and shared interface, with tasks, dependencies and evidence requirements.

## Run it locally

```bash
cargo install cargo-llvm-cov && rustup component add llvm-tools-preview
scripts/coverage-rust.sh coverage/rust/rust.lcov
scripts/coverage-frontend.sh coverage/frontend
python3 scripts/coverage-report.py coverage/site coverage/rust/rust.lcov $(find coverage/frontend -name lcov.info)
xdg-open coverage/site/index.html
```

The instrumented Rust build goes to `target/llvm-cov-target`, apart from the normal `target/`.
