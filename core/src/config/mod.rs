//! Application configuration management

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub mod app_config;
pub mod migration;

pub use app_config::{
	AppConfig, JobLoggingConfig, LogStreamConfig, LoggingConfig, ProxyPairingConfig, ServiceConfig,
	SpacebotConfig,
};
pub use migration::Migrate;

/// Default data directory: `~/.wingdrive` on desktop, platform data dir on mobile.
///
/// Delegates to [`crate::branding`] so an install predating the fork keeps
/// reading from its `WingDrive` directory instead of silently starting empty.
pub fn default_data_dir() -> Result<PathBuf> {
	crate::branding::data_dir()
}

static OWN_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Record where this process keeps its own data, so walks can refuse to
/// index it. A daemon whose data dir sits on an indexed drive otherwise
/// records its own SQLite journals, which are born and deleted fast enough
/// to give one path two identities and poison the batch around it. First
/// caller wins; a process has one data dir.
pub fn mark_own_data_dir(dir: &Path) {
	let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
	let _ = OWN_DATA_DIR.set(canonical);
}

/// Whether a path is inside this process's own data directory.
pub fn is_own_data(path: &Path) -> bool {
	OWN_DATA_DIR.get().is_some_and(|own| path.starts_with(own))
}

/// User preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
	pub theme: String,    // "light", "dark", "system"
	pub language: String, // ISO 639-1 code
}

impl Default for Preferences {
	fn default() -> Self {
		Self {
			theme: "system".to_string(),
			language: "en".to_string(),
		}
	}
}
