//! # Archives
//!
//! Compress and extract archives as jobs, so large archives report progress
//! and can be cancelled. Both write into a hidden temporary path next to the
//! result and rename it into place only when complete, so a cancelled or
//! failed run never leaves a half-written archive or folder that looks real.

pub mod engine;
pub mod format;
pub mod job;

pub use format::ArchiveFormat;
pub use job::{
	ArchiveCompressAction, ArchiveCompressInput, ArchiveExtractAction, ArchiveExtractInput,
};
