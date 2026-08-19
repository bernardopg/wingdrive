//! Test helper modules for integration tests
//!
//! These helpers are shared across many integration test binaries. Each binary
//! only exercises a subset, so allow dead code to keep the harness buildable
//! without per-test allow attributes.

#![allow(dead_code, unused_imports)]

pub mod event_collector;
pub mod indexing_harness;
pub mod snapshot;
pub mod sync_harness;
pub mod sync_transport;
pub mod test_data;
pub mod test_volumes;

pub use event_collector::*;
pub use indexing_harness::*;
pub use snapshot::*;
pub use sync_harness::*;
pub use sync_transport::*;
pub use test_data::*;
