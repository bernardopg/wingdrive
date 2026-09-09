//! Filesystem Watcher Service
//!
//! Wraps `wing-fs-watcher` for platform-agnostic filesystem event detection.
//!
//! ## Architecture
//!
//! - **FsWatcherService**: Detects filesystem changes, emits events via broadcast channel
//! - **Handlers** (in `ops/indexing/handlers/`): Subscribe to events and route them
//!

mod service;

pub use crate::ops::indexing::handlers::LocationMeta;
pub use service::{FsWatcherService, FsWatcherServiceConfig};
