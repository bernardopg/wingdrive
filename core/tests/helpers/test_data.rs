//! Test data directory management with automatic cleanup and snapshot support

use super::snapshot::SnapshotManager;
use std::path::{Path, PathBuf};

/// Manages test data directories with automatic cleanup and optional snapshot support
pub struct TestDataDir {
	test_name: String,
	temp_path: PathBuf,
	snapshot_manager: Option<SnapshotManager>,
}

impl TestDataDir {
	/// Create new test data directory in system temp location
	///
	/// Directory structure:
	/// ```
	/// /tmp/wingdrive-test-{test_name}/
	/// ├── core_data/       # Core database and state
	/// ├── locations/       # Test file locations
	/// └── logs/            # Test execution logs
	/// ```
	///
	/// Snapshots are enabled if WING_TEST_SNAPSHOTS=1 environment variable is set.
	pub fn new(test_name: impl Into<String>) -> anyhow::Result<Self> {
		Self::with_mode(test_name, false)
	}

	/// Create test data directory with filesystem watcher support
	///
	/// Uses home directory instead of temp on macOS because temp directories
	/// don't reliably deliver filesystem events. This is required for tests
	/// that use the filesystem watcher.
	pub fn new_for_watcher(test_name: impl Into<String>) -> anyhow::Result<Self> {
		Self::with_mode(test_name, true)
	}

	fn with_mode(test_name: impl Into<String>, use_home_for_watcher: bool) -> anyhow::Result<Self> {
		let test_name = test_name.into();

		// Choose base directory based on watcher requirements
		let temp_base = if use_home_for_watcher {
			// Use home directory for watcher support (macOS temp doesn't deliver events)
			if cfg!(windows) {
				std::env::var("USERPROFILE").unwrap_or_else(|_| {
					std::env::var("TEMP").unwrap_or_else(|_| "C:\\temp".to_string())
				})
			} else {
				std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
			}
		} else {
			// Use temp directory for regular tests
			if cfg!(windows) {
				std::env::var("TEMP").unwrap_or_else(|_| "C:\\temp".to_string())
			} else {
				"/tmp".to_string()
			}
		};

		let prefix = if use_home_for_watcher {
			".wingdrive_test_"
		} else {
			"wingdrive-test-"
		};
		let temp_path = PathBuf::from(temp_base).join(format!(
			"{prefix}{}-{}",
			test_name,
			uuid::Uuid::new_v4()
		));
		std::fs::create_dir(&temp_path)?;

		// Create standard subdirectories
		std::fs::create_dir_all(temp_path.join("core_data"))?;
		std::fs::create_dir_all(temp_path.join("locations"))?;
		std::fs::create_dir_all(temp_path.join("logs"))?;

		// Check if snapshots are enabled
		let snapshot_enabled = std::env::var("WING_TEST_SNAPSHOTS")
			.map(|v| v == "1" || v.to_lowercase() == "true")
			.unwrap_or(false);

		let snapshot_manager = if snapshot_enabled {
			Some(SnapshotManager::new(&test_name, &temp_path)?)
		} else {
			None
		};

		Ok(Self {
			test_name,
			temp_path,
			snapshot_manager,
		})
	}

	/// Get path to temp directory root
	pub fn path(&self) -> &Path {
		&self.temp_path
	}

	/// Get path for core data (database, preferences, etc.)
	pub fn core_data_path(&self) -> PathBuf {
		self.temp_path.join("core_data")
	}

	/// Get path for test locations
	pub fn locations_path(&self) -> PathBuf {
		self.temp_path.join("locations")
	}

	/// Get path for test logs
	pub fn logs_path(&self) -> PathBuf {
		self.temp_path.join("logs")
	}

	/// Check if snapshots are enabled
	pub fn snapshots_enabled(&self) -> bool {
		self.snapshot_manager.is_some()
	}

	/// Get snapshot manager (if snapshots enabled)
	pub fn snapshot_manager(&self) -> Option<&SnapshotManager> {
		self.snapshot_manager.as_ref()
	}

	/// Get mutable snapshot manager (if snapshots enabled)
	pub fn snapshot_manager_mut(&mut self) -> Option<&mut SnapshotManager> {
		self.snapshot_manager.as_mut()
	}

	/// Get test name
	pub fn test_name(&self) -> &str {
		&self.test_name
	}
}

impl Drop for TestDataDir {
	fn drop(&mut self) {
		// Capture final snapshot if enabled and not already captured
		if let Some(manager) = &mut self.snapshot_manager {
			if !manager.captured() {
				// Use blocking operation in drop
				let _ = manager.capture_final_blocking();
			}
		}

		// Clean up temp directory
		// On Windows, SQLite file locks may persist even after shutdown, causing
		// removal to fail. Since tests use unique directories (via atomic counter),
		// we can safely ignore cleanup failures. Windows will clean temp directories
		// periodically.
		if let Err(e) = std::fs::remove_dir_all(&self.temp_path) {
			#[cfg(windows)]
			{
				// Silently ignore on Windows - file locks are expected
				let _ = e;
			}
			#[cfg(not(windows))]
			{
				eprintln!(
					"Warning: Failed to clean up test directory {:?}: {}",
					self.temp_path, e
				);
			}
		}
	}
}
