# WingDrive Core v2 Development Guide

## Quick Start

### Development Workflow

1. Start daemon: `cargo run --bin wing-daemon`
2. Make code changes
3. Run tests: `cargo test`
4. Rebuild and restart: `cargo run --bin wing -- restart`
5. Test via CLI: `cargo run --bin wing -- <command>`

### Common Commands

```bash
cargo build                              # Build the project
cargo test                               # Run all tests
cargo test <test_name>                   # Run specific test
cargo clippy                             # Lint code
cargo fmt                                # Format code
cargo run --bin wing -- <command>      # Run CLI (binary is `wing`)
```

### Common Mistakes

- Running `wingdrive` instead of `wing-cli` (the binary name is `wing-cli`)
- Forgetting to restart daemon after rebuilding
- Using `println!` instead of `tracing` macros (`info!`, `debug!`, etc)
- Implementing `Wire` manually instead of using `register_*` macros
- Blocking the async runtime with synchronous I/O operations
- Hardcoding user-visible English text in the UI instead of an i18n key (see [Internationalization](#internationalization-i18n))

### Quick tips

- On frontend apps, such as the interface in React, you must ALWAYS ensure type-safety based on the auto generated TypeScript types from `ts-client`. Never cast to as any or redefine backend types. our hooks are typesafe with correct input/output types, but sometimes you might need to access types directly from the `ts-client`.
- If you have changed types on the backend that are public to the frontend (have `Type` derive), then you must regenerate the types using `cargo run --bin generate_typescript_types`
- Read the `.mdx` files in /docs for context on any part of the app, they are kept up to date.
-

## Architecture Overview

WingDrive uses daemon-client architecture. A single daemon process manages core functionality. Multiple clients (CLI, GraphQL server, desktop app) connect via Unix domain sockets.

### CQRS and DDD Pattern

- **Domain** (`src/domain/`): Core data structures and business logic (nouns)
- **Operations** (`src/ops/`): Actions and queries (verbs)
- **Actions**: State-changing operations (writes)
- **Queries**: Data retrieval without state changes (reads)

### Feature Module Structure

Each feature lives in its own module under `src/ops/`. Example: `src/ops/files/share`

```
src/ops/files/share/
├── action.rs      # State-changing logic
├── input.rs       # Action input structures
├── output.rs      # Action output structures
└── job.rs         # Long-running job implementation (if needed)
```

Complete feature example:

```rust
// src/ops/files/share/input.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareFileInput {
    pub file_id: i32,
    pub recipient: String,
}

// src/ops/files/share/output.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct ShareFileOutput {
    pub share_id: String,
    pub url: String,
}

// src/ops/files/share/action.rs
use super::{ShareFileInput, ShareFileOutput};

pub struct ShareFileAction;

crate::register_library_action!(ShareFileAction, "files.share");

impl Action for ShareFileAction {
    type Input = ShareFileInput;
    type Output = ShareFileOutput;

    async fn run(input: Self::Input, ctx: &ActionContext) -> Result<Self::Output> {
        // Implementation
    }
}
```

## Communication Architecture

WingDrive supports multiple communication patterns for different platforms and use cases.

### Daemon-Client Communication (Tauri Desktop, CLI, Web)

The Tauri desktop app, CLI, and web interface connect to a daemon process via Unix domain sockets (or WebSockets for web). Communication uses JSON-RPC 2.0 with Wire method strings.

**Registration Macros:**

Never implement `Wire` manually. Use registration macros:

```rust
// Queries
crate::register_query!(NetworkStatusQuery, "network.status");
// Generates: "query:network.status"

// Library Actions
crate::register_library_action!(FileCopyAction, "files.copy");
// Generates: "action:files.copy.input"

// Core Actions
crate::register_core_action!(LibraryCreateAction, "libraries.create");
// Generates: "action:libraries.create.input"
```

**Registry System:**

The `inventory` crate collects operations at compile time. When you use `register_query!` or `register_library_action!`, the operation automatically appears in global `QUERIES` and `ACTIONS` hashmaps at startup. You never manually register operations.

Location: `core/src/infra/wire/registry.rs`

### Tauri Desktop Development

The Tauri app (`apps/tauri/`) is the primary desktop application for WingDrive. It connects to the daemon via the TypeScript client.

**Development Workflow:**

```bash
# Install dependencies
bun install

# Run Tauri app in dev mode (auto-starts daemon)
cd apps/tauri
bun run tauri:dev

# Build for production
bun run tauri:build
```

**TypeScript Client:**

The TypeScript client (`packages/ts-client/`) is auto-generated from Rust types using Specta:

```bash
# Generate TypeScript types
cargo run --bin generate_typescript_types
```

**Output:** `packages/ts-client/src/generated.ts`

**Architecture:**

```
Tauri App (React)
    ↓
@wingdrive/ts-client (TypeScript)
    ↓
Daemon (Unix Socket / IPC)
    ↓
RpcServer (Rust)
    ↓
Operation Registry
```

### Internationalization (i18n)

Every user-visible string in the UI is translated. A change that adds or edits UI text is not done until
the text goes through i18n with both English and Brazilian Portuguese entries. Reviews reject hardcoded
strings.

- **Stack:** i18next with react-i18next, set up in `packages/interface/src/i18n/`. English (`en`) is the
  source language and `pt` (Brazilian Portuguese) is the target. The daemon stores the choice in
  `preferences.language` as a two-letter code, which is why the folder is `pt`, not `pt-BR`.
- **Catalogs:** `packages/interface/src/i18n/locales/<lang>/<namespace>.json`. The namespaces are
  `settings`, `explorer`, `inspector`, `dialogs`, `navigation` and `common`. Add keys to the namespace that
  owns the component. A new namespace must also be registered in `i18n/index.ts`.
- **What to translate:** JSX text, labels, tooltips, `title`, `placeholder`, `aria-label`, `alt`, menu and
  context-menu items, toasts, dialogs, confirmations and empty states.
- **What not to translate:** logs, errors that never reach the user, identifiers, protocol and enum
  values, paths, URLs, keyboard symbols, and brand names (WingDrive, Wingbot, Spacebot).
- **In components:** `const { t } = useTranslation("explorer");` then `t("contextMenu.copy")`.
  `apps/tauri/src` imports `useTranslation` and `i18n` from `@wingdrive/interface`.
- **Outside React render** (module constants, registries, callbacks built once): never call `t` at module
  load. Resolve at call time with `i18n.t("explorer:contextMenu.copy")`, or store the key and translate
  where it is rendered. Add `t` to the dependencies of any `useMemo`/`useCallback` that builds labels.
- **Interpolation and plurals:** `t("copied", { count })` with `{{count}}` in JSON, and `_one` / `_other`
  suffixes for plurals. Never concatenate translated fragments into a sentence.
- **Parity:** `en` and `pt` files must have the same keys. Portuguese uses natural Brazilian wording with
  correct diacritics. Keys are nested and named after the component (`emptyState.title`), and JSON uses
  tab indentation.
- **Crowdin:** project `wingdrive`, configured in `crowdin.yml`. `.github/workflows/crowdin.yml` uploads the
  `en` sources on every push to `main` that changes them and opens a `chore(i18n)` pull request with the
  translations. To sync locally, use `bun crowdin upload sources` and `bun crowdin download` with
  `CROWDIN_PERSONAL_TOKEN` and `CROWDIN_PROJECT_ID` set. Do not edit Crowdin-managed `pt` strings by hand
  in the same pull request as a Crowdin sync.

### Native Prototypes (iOS, macOS)

**Note:** iOS and macOS apps are experimental prototypes, not production apps.

Native prototypes embed the core directly as a library via FFI rather than connecting to a daemon. These are located in `apps/ios/` and `apps/macos/` but are private and not documented for public use.

**Swift Client Generation:**

For the prototypes, Swift types can be generated:

```bash
cargo run --bin generate_swift_types
```

Output: `packages/swift-client/Sources/WingDriveClient/`

### Extension System (WASM)

Extensions run as sandboxed WASM modules that interact with WingDrive core via host functions. Extensions are distributed as compiled `.wasm` files.

**Architecture:**

```
Extension.wasm (compiled Rust)
    ↓
wingdrive-sdk (Rust crate)
    ↓
Host Functions (FFI boundary)
    ↓
Core (VDFS, Jobs, AI, etc.)
```

**Key Components:**

**SDK Location:** `crates/sdk/`

- High-level Rust API abstracting FFI details
- Procedural macros for extension definition
- Type-safe job, model, and action builders

**Extension Development:**

Extensions use procedural macros to minimize boilerplate:

```rust
use wingdrive_sdk::prelude::*;

#[extension(
    id = "test-extension",
    name = "Test Extension",
    version = "0.1.0",
    jobs = [test_counter],
)]
struct TestExtension;

#[derive(Serialize, Deserialize, Default)]
pub struct CounterState {
    pub current: u32,
    pub target: u32,
    pub processed: Vec<String>,
}

#[job(name = "counter")]
fn test_counter(ctx: &JobContext, state: &mut CounterState) -> Result<()> {
    ctx.log(&format!("Starting counter (current: {}, target: {})",
        state.current, state.target));

    while state.current < state.target {
        if ctx.check_interrupt() {
            ctx.checkpoint(state)?;
            return Err(Error::OperationFailed("Interrupted".into()));
        }

        state.current += 1;
        ctx.report_progress(
            state.current as f32 / state.target as f32,
            &format!("Counted {}/{}", state.current, state.target),
        );

        if state.current % 10 == 0 {
            ctx.checkpoint(state)?;
        }
    }

    Ok(())
}
```

**Host Functions:**

Extensions import minimal FFI functions:

```rust
#[link(wasm_import_module = "wingdrive")]
extern "C" {
    fn wingdrive_log(level: u32, msg_ptr: *const u8, msg_len: usize);
    fn register_job(
        job_name_ptr: *const u8,
        job_name_len: u32,
        export_fn_ptr: *const u8,
        export_fn_len: u32,
        resumable: u32,
    ) -> i32;
}
```

**Building Extensions:**

```bash
# From extension directory
cargo build --target wasm32-unknown-unknown --release

# Output: target/wasm32-unknown-unknown/release/extension_name.wasm
```

**Extension Capabilities:**

Extensions can define:

- Models: Data structures stored in `models` table (content-scoped, standalone, or entry-scoped)
- Jobs: Long-running resumable operations
- Actions: User-invoked operations with preview-commit workflow
- Agents: Autonomous logic with memory and event handling
- UI: Custom views via `ui_manifest.json`

**Example Use Cases:**

- Photos extension: Face detection, scene tagging, album organization
- Finance extension: Receipt extraction, expense tracking
- Research extension: Citation extraction, knowledge graphs

**Key Benefits:**

- Single `.wasm` file works on all platforms
- True sandboxing (WASM isolation)
- Resumable jobs with checkpointing
- Type-safe API with procedural macros
- No core modifications needed for new features

**Documentation:**

- `/docs/sdk/sdk.md` - Complete SDK specification and API reference
- `extensions/test-extension/` - Working example extension
- `crates/sdk/` - SDK implementation
- `crates/sdk-macros/` - SDK procedural macros

**Status:** SDK implementation in progress. Test extension compiles to WASM successfully. Core integration for loading and executing WASM modules is next phase.

## Code Standards

### Import Organization

Group imports with blank lines between groups:

```rust
// Standard library
use std::path::PathBuf;
use std::sync::Arc;

// External crates
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

// Local modules
use crate::domain::library::Library;
use crate::ops::Action;
```

### Naming Conventions

- Functions/variables: `snake_case`
- Types: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`

### Error Handling

Use `Result<T, E>` for all fallible operations. Use `thiserror` for custom errors, `anyhow` for application errors.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShareError {
    #[error("File not found: {0}")]
    FileNotFound(i32),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
}

pub async fn share_file(id: i32) -> Result<String, ShareError> {
    let file = find_file(id).await.ok_or(ShareError::FileNotFound(id))?;
    // Implementation
    Ok(share_url)
}
```

### Async Code

- Use `async/await` syntax
- Prefer `tokio` primitives (`tokio::sync::RwLock`, `tokio::spawn`)
- Avoid blocking operations (use `tokio::fs` not `std::fs`)
- Use `tokio::task::spawn_blocking` for CPU-intensive work

### Resumable Jobs

Store job state within the job struct. Use `#[serde(skip)]` for non-persistent fields.

```rust
#[derive(Serialize, Deserialize)]
pub struct FileCopyJob {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub copied_files: Vec<PathBuf>,  // Persisted for resumability

    #[serde(skip)]
    pub progress_tx: Option<tokio::sync::mpsc::Sender<Progress>>,  // Not persisted
}

impl Job for FileCopyJob {
    async fn run(&mut self, ctx: &JobContext) -> Result<()> {
        ctx.log().info("Starting file copy job");

        for file in &self.files_to_copy {
            if self.copied_files.contains(file) {
                continue;  // Skip already copied files on resume
            }

            copy_file(file).await?;
            self.copied_files.push(file.clone());
        }

        Ok(())
    }
}
```

### Documentation

**Core principle:** Explain WHY, not WHAT. Keep comments as short as possible. One sentence explaining rationale beats a paragraph restating code.

**Module docs (`//!`):**
- Add a title with `#` for the module name
- Explain what the module does in plain language (not bullet points)
- Include design rationale naturally in prose
- Add runnable code examples showing usage

````rust
//! # File Sharing System
//!
//! `core::ops::files::share` provides temporary file sharing via signed URLs.
//! Share links expire after 7 days by default to prevent indefinite access to
//! private files. UUID v5 deterministic IDs ensure the same file generates
//! consistent share URLs across devices without coordination.
//!
//! ## Example
//! ```rust,no_run
//! use wingdrive_core::ops::files::share::{ShareFileAction, ShareFileInput};
//!
//! let input = ShareFileInput { file_id: 123, recipient: "user@example.com" };
//! let output = ShareFileAction::run(input, &ctx).await?;
//! ```
````

**Function docs (`///`):**
- First line: brief one-liner
- Second paragraph: explain design rationale and why this exists
- Document error handling philosophy when relevant
- Explain non-obvious behavior and platform differences

```rust
/// Creates a share link with automatic expiration.
///
/// Share links use signed JWTs so the daemon can validate them without
/// database lookups on every request. Expiration is enforced server-side
/// to prevent timezone manipulation. Recipients without library access
/// get read-only access to the specific file only.
///
/// Returns `ShareError::PermissionDenied` if the file is private and
/// the recipient isn't a library member. The share is still created
/// but marked inactive for audit logging.
pub async fn share_file(input: ShareFileInput) -> Result<ShareFileOutput>
```

**Inline comments:**
- Delete comments that restate obvious code
- Explain WHY for decisions, not WHAT the code does
- Use one sentence when possible
- Only expand for truly non-obvious consequences

```rust
// Good: explains WHY
// Lowercase for case-insensitive search matching.
let ext = path.extension().map(|e| e.to_lowercase());

// Bad: restates code
// Extract file extension and convert to lowercase
let ext = path.extension().map(|e| e.to_lowercase());

// Good: explains consequence
// Preserve ephemeral UUIDs so tags attached during browsing survive promotion to managed location.
let uuid = ephemeral_cache.get(path).unwrap_or_else(|| Uuid::new_v4());

// Bad: verbose explanation of obvious behavior
// UUID assignment strategy:
// 1. First check if there's an ephemeral UUID
// 2. If not, generate a new one
let uuid = ephemeral_cache.get(path).unwrap_or_else(|| Uuid::new_v4());
```

**Error handling comments:**
Explain strategy and recovery, not just "log and continue".

```rust
// Good: explains recovery
// Best-effort: continue with remaining moves, stale paths cleaned up on next reindex.
Err(e) => ctx.log(format!("Failed to move: {}", e)),

// Bad: states the obvious
// Log error but continue
Err(e) => ctx.log(format!("Failed to move: {}", e)),
```

**Platform-specific comments:**
Explain consequences, not implementation blockers.

```rust
// Good: explains why and fallback
#[cfg(windows)]
pub fn get_inode(_metadata: &std::fs::Metadata) -> Option<u64> {
    // Windows file indices are unstable across reboots; fall back to path-only matching.
    None
}

// Bad: over-explains implementation details
#[cfg(windows)]
pub fn get_inode(_metadata: &std::fs::Metadata) -> Option<u64> {
    // Windows doesn't have inodes.
    // The method `file_index()` is unstable (issue #63010).
    // Returning None is safe as the field is Optional.
    None
}
```

**Never use:**
- Placeholder comments ("for now", "TODO: extract this later")
- Markdown formatting (`**bold**`, `_italic_`) in code comments
- ASCII diagrams (put those in `/docs/` if needed)
- Section divider comments (`// ========== Section ==========`)
- Comments explaining removed code during refactors

Track future work in GitHub issues, not code comments.

### Formatting

Run `cargo fmt` before committing. Tabs for indentation. No emojis.

## Logging

### Setup

Use `tracing_subscriber` in main or examples:

```rust
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("wing_core=info"))
        )
        .init();
}
```

## Writing Style

This applies to all documentation, code comments, and design documents.

Use clear, simple language. Write short, impactful sentences. Use active voice. Focus on practical, actionable information.

Address the reader directly with "you" and "your". Support claims with data and examples when possible.

Avoid these constructions:

- Em dashes (use commas or periods)
- "Not only this, but also this"
- Metaphors and cliches
- Generalizations
- Setup language like "in conclusion"
- Unnecessary adjectives and adverbs
- Emojis, hashtags, markdown formatting in prose

Avoid these words:
comprehensive, delve, utilize, harness, realm, tapestry, unlock, revolutionary, groundbreaking, remarkable, pivotal

### Macros

Use `tracing` macros, never `println!`:

```rust
use tracing::{info, warn, error, debug};

info!("Server started on port {}", port);
debug!(file_id = %id, "Processing file");
warn!(error = %e, "Retrying operation");
error!("Failed to connect to database");
```

### Job Logging

Use `ctx.log()` in job implementations for automatic `job_id` tagging:

```rust
impl Job for MyJob {
    async fn run(&mut self, ctx: &JobContext) -> Result<()> {
        ctx.log().info("Job started");
        ctx.log().debug!(progress = %self.progress, "Processing");
        Ok(())
    }
}
```

### Log Levels

- `debug`: Detailed flow for troubleshooting
- `info`: User-relevant events (server start, job completion)
- `warn`: Recoverable issues (retry, fallback)
- `error`: Failures requiring attention

### Environment Control

Use `RUST_LOG` environment variable:

```bash
RUST_LOG=debug cargo run --bin wing
RUST_LOG=wing_core=trace cargo run
RUST_LOG=wing_core::ops=debug cargo run
```

## Testing

### Test Organization

- Unit tests: Colocated in `#[cfg(test)]` modules
- Integration tests: `tests/` directory at crate root

```rust
// src/ops/files/share/action.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_share_file() {
        let input = ShareFileInput {
            file_id: 1,
            recipient: "test@example.com".to_string(),
        };

        let output = share_file(input).await.unwrap();
        assert!(!output.share_id.is_empty());
    }
}
```

### Running Tests

```bash
cargo test                    # All tests
cargo test test_share_file    # Specific test
cargo test --lib              # Library tests only
cargo test -- --nocapture     # Show output
```

## Before every push

CI is the last gate, not the place to find errors: a Rust run costs about an hour.
Run the same gates locally first, on the warm `target/` directory:

```bash
just ci-local        # scripts/ci-local.sh; skips Rust steps when no Rust-relevant file changed
just ci-local --all  # force the Rust steps
```

Rules for agents and contributors:

- Push only after `just ci-local` passes. Fix failures locally, then push once.
- Push to `origin` (`bernardopg/wingdrive`) and open pull requests there
  (`gh pr create --repo bernardopg/wingdrive`). The `fork` remote is a different repository.
- After a stacked pull request merges into its base branch, merge `origin/main` into the base.
  `.github/workflows/ci.yml` usually conflicts on the `bun test` list; keep both sides.
- Never push to "see what CI says", and never push a series of small fix commits to
  iterate against CI. Reproduce the CI failure locally (same `cargo test` or `bun test`
  command from `.github/workflows/ci.yml`), fix it, rerun, then push.
- Reuse the local `target/`; do not `cargo clean` and do not use a separate `--target-dir`
  for checks. Run one cargo command at a time (concurrent cargo runs block on the build lock).
- After a push, check CI once when it finishes (`gh pr checks <n> --watch -R bernardopg/wingdrive`).
  Rerun only for infrastructure failures (runner timeout, network); a test failure is fixed locally first.
- Changes limited to `.tasks/`, `docs/`, Markdown, `packages/` or the frontend skip the Rust CI job;
  `just ci-local` applies the same filter.
- Changes limited to one app's Rust crates (`apps/tauri/`, `apps/server/`, `apps/cli/`,
  `apps/mobile/modules/`) lint and test only that app. `scripts/ci-scope.sh` picks the scope and
  `scripts/ci-rust.sh` runs it, in CI and in `just ci-local`.
- A push to `main` runs only the gates its own changes need, and skips every job when it merged a pull
  request whose head has the same tree and already passed CI (`scripts/ci-changes.sh`). Keep a branch up
  to date with `main` before merging so the merge commit matches its tested tree. A nightly scheduled
  run on `main` runs every gate and refreshes the Rust cache that pull requests restore.
- CodeQL analyzes Rust (about 35 minutes) only when a change touches `*.rs`, a `Cargo.toml`,
  `Cargo.lock` or the CodeQL config; the weekly schedule always includes it.
- A release tag on a commit that `main`'s CI already checked reuses that run. Tag a commit on `main`
  after its CI finishes; a tag on any other commit makes the release dispatch a new CI run on the tag.
- Typecheck with `bun run typecheck` (or `just typecheck`). It runs `bun check` (Bun 1.4.3+) on
  tauri, web, interface, ts-client, ai and primitives. `bun check` reports the same errors as
  `tsc` 7 in about a third of the time. Your editor keeps using the `typescript` package for its
  language server.
- The desktop shell (`apps/tauri/src-tauri`, `wing-tauri-core`) must not depend on `wing-core`.
  Put anything it needs from the core in a small crate under `crates/`, as `wing-branding` does.

## Repository Rules

`main` only changes through pull requests. Its ruleset (`.github/rulesets/main.json`) requires the CI
jobs, the TruffleHog secret scan and CodeQL merge protection, and blocks force-pushes and deletion.
`v*` tags cannot be moved or deleted. Change a rule by editing the JSON in a pull request, then run
`scripts/github-settings.sh` as an admin after it merges.

## Naming

Never name anything "Spacedrive" or with an `sd`/`sd-` prefix: crates, binaries, packages, scripts, functions, types, environment variables, or user-facing text. Use `wingdrive` or `wing`. The only exception is legally required upstream attribution. Existing `sd` names are being removed under `FORK-003`; do not add new ones.

## Task Tracking

WingDrive uses a file-based task system in `/.tasks/` to track features, epics, and development work. All task files are version-controlled alongside the code.

Planning follows the hierarchy roadmap, milestone, epic, task, with two-week sprints. Read `.tasks/PLANNING.md` for the method and Definition of Done, `.tasks/ROADMAP.md` for milestone order, and `.tasks/sprints/` for the current sprint. Tasks set `parent` (epic), `milestone` (`M1`, `M2`, ...), and, once committed to a sprint, `sprint` (`S01`, ...).

### When to Create Tasks

Create tasks for work that:

- Introduces a new feature or capability
- Refactors a significant system or module
- Fixes a bug requiring architectural changes
- Implements a whitepaper specification

Do not create tasks for:

- Routine code formatting or style fixes
- Trivial bug fixes (single line changes)
- Documentation updates to existing features
- Dependency version bumps

### Task Structure

Each task is a Markdown file: `CATEGORY-###-title-slug.md`

```yaml
---
id: CORE-042
title: "Implement file sharing API"
status: "In Progress"
assignee: "james"
priority: "High"
milestone: "M1"
sprint: "S01"
tags: ["core", "networking"]
whitepaper: "Section 4.2" # And/or design_doc: DESIGN_DOC_NAME.md
---

## Description
Brief overview of what needs to be done and why.

## Implementation Steps
- [ ] Create share action in src/ops/files/share
- [ ] Add database schema for shares table
- [ ] Implement expiration logic

## Acceptance Criteria
- Share links work across all platforms
- Expired shares return 404
- Tests cover edge cases
```

### Managing Tasks

```bash
# List your active tasks
cargo run -p task-validator -- list --assignee "yourname" --status "In Progress"

# List high priority tasks
cargo run -p task-validator -- list --priority "High" --sort-by id

# Show the current sprint board or a milestone backlog
cargo run -p task-validator -- list --sprint S01
cargo run -p task-validator -- list --milestone M1 --status "To Do"

# Validate before committing (automatic via git hook)
cargo run -p task-validator -- validate
```

### Task Lifecycle

1. Create task file in `/.tasks/` with `status: "To Do"`
2. Update status to `"In Progress"` when you start work
3. Complete implementation and tests
4. Update status to `"Done"` and commit

Full documentation: `/docs/core/task-tracking.md`

## Debugging

### Log Files

Job logs live in the `job_logs` directory in the data folder root.

### Daemon Restart

After rebuilding, restart the daemon to use the latest code:

```bash
cargo build
cargo run --bin wing -- restart
```

### Mobile App

Never iterate with release APKs. Use the debug dev client and Metro: `just mobile-emulator`,
`just mobile-install-android`, `just dev-mobile`. Full loop in `docs/core/mobile-development.mdx`
and the agent skill `.agents/skills/wingdrive-mobile-dev/SKILL.md`.

### Checking UI Changes at Runtime

Typecheck and unit tests miss rendering bugs, so check Explorer and sidebar changes in the web UI
against an isolated daemon before calling them done. This never touches the author's daemon or
library:

```bash
(cd apps/web && bun run build)
cargo build --bin wing-server --bin wing-daemon
S=/tmp/wing-check   # any scratch folder
HOME=$S/home XDG_CONFIG_HOME=$S/home/.config ./target/debug/wing-server \
  --data-dir $S/data --instance check --host 127.0.0.1 --port 18089 --auth disabled
```

Drive `http://127.0.0.1:18089/` with a browser (Chrome DevTools MCP, isolated context). Stop the
server by PID: `pkill -f` with a pattern that matches its own command line kills your shell. The web
UI has no `homeDir`, `runAsAdmin` or service menus; those need the Tauri build.

### Frontend Pitfalls

- TanStack Table's legacy `getCoreRowModel` rebuilds only when `data` changes identity. Pass a new
  array when sub-rows change (see `ListView/useTable.tsx`).
- `bun test` resolves `./folderTree` to `FolderTree.tsx` in the same folder. Do not give two files
  names that differ only in case.
- The CSP's `connect-src` does not allow `asset:`. Fetch file bytes through
  `platform.getMediaSrc` (the loopback stream URL), as Quick Preview does.
- `useLibraryQueries` subscribes to no change events and keys queries without the `query:` prefix
  that `useNormalizedQuery` uses. Refresh its results yourself, as `useExpandedFolderEvents` does for
  expanded list rows.

### Verbose Logging

```bash
RUST_LOG=debug cargo run --bin wing-daemon
RUST_LOG=wing_core::jobs=trace cargo run
```

## Documentation Locations

- Core architecture: `/docs/core/`
- Design docs and RFCs: `/docs/core/design/`
- Application docs: `/docs/`
- Daemon details: `/docs/core/daemon.md`
- Task tracking: `/docs/core/task-tracking.md`


<!-- headroom:memory-instructions -->
## Memory

Use the `headroom_memory` MCP server for persistent cross-session knowledge.

**Before** answering questions about prior decisions, conventions, project context,
architecture, user preferences, org info, codenames, debugging history, or anything
from past sessions — call `memory_search` first.

**After** making durable decisions, discovering conventions, or learning important
facts — call `memory_save` to persist them for future sessions.

Memory is your first source of truth for anything not visible in the current conversation.
