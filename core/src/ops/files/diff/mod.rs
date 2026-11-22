//! Path intersection / smart diff between two directories.
//!
//! Compares two paths and returns what is missing, extra, or modified.

mod action;
mod input;
mod output;

pub use action::PathDiffAction;
pub use input::{DiffStrategy, PathDiffInput};
pub use output::{CompareStatus, DiffEntry, PathDiffResult};
