//! # Brand Identity and Legacy Path Resolution
//!
//! `core::branding` owns every on-disk name the product answers to. WingDrive is
//! a fork of Spacedrive, so any install predating the rename keeps its config,
//! libraries and device identity under `Spacedrive` paths.
//!
//! Every resolver here prefers the WingDrive location but falls back to the
//! legacy one when it already exists on disk. An upgrade therefore keeps working
//! without moving a single byte, and a fresh install never touches the legacy
//! name. Moving data would risk orphaning libraries for no benefit, since the
//! only thing the rename must satisfy is the trademark clause on user-facing
//! branding.
//!
//! ## Example
//! ```rust,no_run
//! use sd_core::branding;
//!
//! // `~/.wingdrive`, or `~/.spacedrive` when only the legacy install exists.
//! let dir = branding::data_dir()?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

/// User-facing product name.
pub const APP_NAME: &str = "WingDrive";

/// Product name used before the fork, still present on existing installs.
pub const LEGACY_APP_NAME: &str = "Spacedrive";

/// OS keychain service name for the device key.
pub const KEYRING_SERVICE: &str = "WingDrive";

/// Keychain service used before the fork.
pub const LEGACY_KEYRING_SERVICE: &str = "Spacedrive";

/// Reverse-DNS prefix, matching the Tauri bundle identifier.
pub const BUNDLE_PREFIX: &str = "com.wingdrive";

/// Reverse-DNS prefix used before the fork.
pub const LEGACY_BUNDLE_PREFIX: &str = "com.spacedrive";

const DATA_DIR_NAME: &str = ".wingdrive";
const LEGACY_DATA_DIR_NAME: &str = ".spacedrive";

const LOWER_APP_NAME: &str = "wingdrive";
const LEGACY_LOWER_APP_NAME: &str = "spacedrive";

/// Picks `preferred` unless only `legacy` exists on disk.
///
/// Existence is the tiebreaker rather than a stored migration flag, because a
/// flag can desync from reality when a user copies a home directory between
/// machines. When neither path exists this returns `preferred`, so first runs
/// always create WingDrive paths.
pub fn resolve_with_legacy(preferred: impl AsRef<Path>, legacy: impl AsRef<Path>) -> PathBuf {
	let preferred = preferred.as_ref();
	let legacy = legacy.as_ref();

	if !preferred.exists() && legacy.exists() {
		return legacy.to_path_buf();
	}

	preferred.to_path_buf()
}

/// Returns true when the resolved path is a legacy Spacedrive location.
///
/// Callers use this to surface an explicit "running against a legacy install"
/// notice instead of silently reading from a differently branded directory.
pub fn is_legacy_path(path: impl AsRef<Path>) -> bool {
	path.as_ref().components().any(|component| {
		let part = component.as_os_str().to_string_lossy();
		part == LEGACY_DATA_DIR_NAME
			|| part == LEGACY_APP_NAME
			|| part == LEGACY_LOWER_APP_NAME
			|| part == LEGACY_BUNDLE_PREFIX
	})
}

/// Root data directory, creating it when absent.
///
/// Desktop uses a dotfile in `$HOME` while mobile uses the platform data dir,
/// because iOS and Android sandbox the home directory away from the app.
pub fn data_dir() -> Result<PathBuf> {
	let dir = data_dir_path()?;
	std::fs::create_dir_all(&dir)?;
	Ok(dir)
}

/// Resolves the data directory without creating it.
///
/// Split out so callers can probe the location during diagnostics without the
/// side effect of materializing an empty directory tree.
pub fn data_dir_path() -> Result<PathBuf> {
	#[cfg(not(any(target_os = "ios", target_os = "android")))]
	{
		let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;
		Ok(resolve_with_legacy(
			home.join(DATA_DIR_NAME),
			home.join(LEGACY_DATA_DIR_NAME),
		))
	}

	#[cfg(any(target_os = "ios", target_os = "android"))]
	{
		let base = dirs::data_dir().ok_or_else(|| anyhow!("Could not determine data directory"))?;
		Ok(resolve_with_legacy(
			base.join(LOWER_APP_NAME),
			base.join(LEGACY_LOWER_APP_NAME),
		))
	}
}

/// Directory holding `device.json` for the current platform.
///
/// macOS uses a reverse-DNS folder under Application Support, Linux follows
/// XDG with a lowercase name, and Windows uses the capitalized product name.
pub fn device_config_dir() -> Option<PathBuf> {
	if cfg!(target_os = "macos") {
		let base = dirs::data_dir()?;
		Some(resolve_with_legacy(
			base.join(BUNDLE_PREFIX),
			base.join(LEGACY_BUNDLE_PREFIX),
		))
	} else if cfg!(target_os = "linux") {
		let base = dirs::config_dir()?;
		Some(resolve_with_legacy(
			base.join(LOWER_APP_NAME),
			base.join(LEGACY_LOWER_APP_NAME),
		))
	} else if cfg!(target_os = "windows") {
		let base = dirs::config_dir()?;
		Some(resolve_with_legacy(
			base.join(APP_NAME),
			base.join(LEGACY_APP_NAME),
		))
	} else {
		None
	}
}

/// Default parent directory for libraries.
///
/// Returns `None` only when the home directory cannot be determined, which
/// callers treat as a hard configuration error rather than falling back to the
/// working directory.
pub fn libraries_dir() -> Option<PathBuf> {
	let home = dirs::home_dir()?;
	Some(resolve_with_legacy(
		home.join(APP_NAME).join("Libraries"),
		home.join(LEGACY_APP_NAME).join("Libraries"),
	))
}

#[cfg(test)]
mod tests {
	use super::*;

	use tempfile::tempdir;

	#[test]
	fn prefers_wingdrive_when_neither_exists() {
		let root = tempdir().unwrap();
		let preferred = root.path().join(".wingdrive");
		let legacy = root.path().join(".spacedrive");

		assert_eq!(resolve_with_legacy(&preferred, &legacy), preferred);
	}

	#[test]
	fn falls_back_to_legacy_when_only_legacy_exists() {
		let root = tempdir().unwrap();
		let preferred = root.path().join(".wingdrive");
		let legacy = root.path().join(".spacedrive");
		std::fs::create_dir_all(&legacy).unwrap();

		assert_eq!(resolve_with_legacy(&preferred, &legacy), legacy);
	}

	#[test]
	fn prefers_wingdrive_when_both_exist() {
		let root = tempdir().unwrap();
		let preferred = root.path().join(".wingdrive");
		let legacy = root.path().join(".spacedrive");
		std::fs::create_dir_all(&preferred).unwrap();
		std::fs::create_dir_all(&legacy).unwrap();

		assert_eq!(resolve_with_legacy(&preferred, &legacy), preferred);
	}

	#[test]
	fn prefers_wingdrive_when_only_it_exists() {
		let root = tempdir().unwrap();
		let preferred = root.path().join(".wingdrive");
		let legacy = root.path().join(".spacedrive");
		std::fs::create_dir_all(&preferred).unwrap();

		assert_eq!(resolve_with_legacy(&preferred, &legacy), preferred);
	}

	#[test]
	fn detects_legacy_paths() {
		assert!(is_legacy_path("/home/user/.spacedrive"));
		assert!(is_legacy_path("/home/user/Spacedrive/Libraries"));
		assert!(is_legacy_path("/home/user/.config/spacedrive"));
		assert!(is_legacy_path(
			"/Users/u/Library/Application Support/com.spacedrive"
		));
	}

	#[test]
	fn does_not_flag_wingdrive_paths_as_legacy() {
		assert!(!is_legacy_path("/home/user/.wingdrive"));
		assert!(!is_legacy_path("/home/user/WingDrive/Libraries"));
		assert!(!is_legacy_path("/home/user/.config/wingdrive"));
	}

	#[test]
	fn does_not_flag_unrelated_substrings() {
		assert!(!is_legacy_path("/home/user/spacedrive-backup"));
		assert!(!is_legacy_path("/home/myspacedrive/data"));
	}
}
