//! Application configuration management

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
/// reading from its `Spacedrive` directory instead of silently starting empty.
pub fn default_data_dir() -> Result<PathBuf> {
	crate::branding::data_dir()
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
