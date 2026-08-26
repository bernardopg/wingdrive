//! # Legacy Spacedrive Path Adoption
//!
//! Proves that a WingDrive build started against a pre-fork install keeps
//! reading the existing `Spacedrive` directories instead of silently starting
//! from an empty state, which would present as total data loss to the user.
//!
//! `$HOME` is overridden for the whole process, so every scenario lives in one
//! test function. Splitting them would let the test harness run them in
//! parallel and race on the same environment variable.
//!
//! The XDG variables are overridden alongside `$HOME` because `dirs::config_dir`
//! prefers `XDG_CONFIG_HOME` when it is set. Without that the test would read
//! the developer's real config directory.

use std::fs;
use std::path::Path;

use sd_core::branding;
use tempfile::tempdir;

/// Points every home-relative lookup at `home` for the current process.
fn set_home(home: &Path) {
	std::env::set_var("HOME", home);
	std::env::set_var("XDG_CONFIG_HOME", home.join(".config"));
	std::env::set_var("XDG_DATA_HOME", home.join(".local").join("share"));
}

/// Restores an environment variable, removing it when it was previously unset.
fn restore(key: &str, value: Option<std::ffi::OsString>) {
	match value {
		Some(value) => std::env::set_var(key, value),
		None => std::env::remove_var(key),
	}
}

#[test]
fn resolves_paths_against_legacy_and_fresh_installs() {
	let original_home = std::env::var_os("HOME");
	let original_config = std::env::var_os("XDG_CONFIG_HOME");
	let original_data = std::env::var_os("XDG_DATA_HOME");

	// A pre-fork install: only Spacedrive directories exist on disk.
	let legacy_home = tempdir().unwrap();
	fs::create_dir_all(legacy_home.path().join(".spacedrive")).unwrap();
	fs::create_dir_all(legacy_home.path().join("Spacedrive").join("Libraries")).unwrap();
	fs::create_dir_all(legacy_home.path().join(".config").join("spacedrive")).unwrap();
	set_home(legacy_home.path());

	let data_dir = branding::data_dir_path().unwrap();
	assert_eq!(
		data_dir,
		legacy_home.path().join(".spacedrive"),
		"an existing Spacedrive install must keep its data directory"
	);
	assert!(branding::is_legacy_path(&data_dir));

	let libraries = branding::libraries_dir().unwrap();
	assert_eq!(
		libraries,
		legacy_home.path().join("Spacedrive").join("Libraries"),
		"existing libraries must stay reachable without being moved"
	);

	if cfg!(target_os = "linux") {
		assert_eq!(
			branding::device_config_dir().unwrap(),
			legacy_home.path().join(".config").join("spacedrive"),
			"device identity must survive the rename"
		);
	}

	// A fresh install: nothing exists, so WingDrive paths are created.
	let fresh_home = tempdir().unwrap();
	set_home(fresh_home.path());

	let data_dir = branding::data_dir_path().unwrap();
	assert_eq!(
		data_dir,
		fresh_home.path().join(".wingdrive"),
		"a fresh install must never adopt the legacy brand"
	);
	assert!(!branding::is_legacy_path(&data_dir));

	assert_eq!(
		branding::libraries_dir().unwrap(),
		fresh_home.path().join("WingDrive").join("Libraries")
	);

	// data_dir() creates the directory; data_dir_path() must not have.
	assert!(!fresh_home.path().join(".wingdrive").exists());
	let created = branding::data_dir().unwrap();
	assert!(created.is_dir());

	// Once a WingDrive directory exists it wins, even beside a legacy one.
	let both_home = tempdir().unwrap();
	fs::create_dir_all(both_home.path().join(".spacedrive")).unwrap();
	fs::create_dir_all(both_home.path().join(".wingdrive")).unwrap();
	set_home(both_home.path());

	assert_eq!(
		branding::data_dir_path().unwrap(),
		both_home.path().join(".wingdrive"),
		"a completed migration must not fall back to the legacy directory"
	);

	restore("HOME", original_home);
	restore("XDG_CONFIG_HOME", original_config);
	restore("XDG_DATA_HOME", original_data);
}
