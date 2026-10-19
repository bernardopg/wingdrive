//! Directory pruning for OS-level watch registration
//!
//! On Linux every watched directory costs one inotify watch, so a recursive
//! watch over a source tree registers tens of thousands of watches inside
//! build outputs and dependency caches, and can exhaust the kernel's watch
//! limit. Those trees emit a flood of events the daemon never acts on: the
//! indexer's default `no_dev_dirs` and `no_git` rules exclude exactly these
//! directories (`node_modules`, `target`, `.git`, caches), so keeping watches
//! for them only pays cost for noise.
//!
//! The recursive walk in [`FsWatcher`](crate::FsWatcher) consults
//! [`is_generated_dir`] for every directory it descends into. The root of a
//! watch is always watched itself; only subdirectories are pruned.

use std::ffi::OsStr;

/// Directory basenames that consume watches without producing useful events.
///
/// Mirrors the indexer's `No Dev Directories` and `No Git files` rules so a
/// watched tree cannot observe what the indexer excludes by default.
const GENERATED_DIRS: &[&str] = &[
	// Version control internals
	".git",
	// Build outputs and dependency checkouts
	"node_modules",
	"target",
	"dist",
	"build",
	// Build and test caches
	"__pycache__",
	".pytest_cache",
	".mypy_cache",
	".tox",
	".nox",
	".hypothesis",
	".cache",
	".turbo",
	// Editor state
	".idea",
	".vscode",
	".vs",
	// Temporary trees
	"tmp",
	"temp",
	".tmp",
	".temp",
];

/// Whether a directory with this basename should not be watched or expanded.
///
/// The comparison is exact on the basename: `dist` matches `.dist`-free
/// trees but `distro/` still watches, and `Target/` (capital T) does not
/// match either.
pub fn is_generated_dir(name: &OsStr) -> bool {
	match name.to_str() {
		Some(name) => GENERATED_DIRS.contains(&name),
		None => false,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn prunes_generated_directories() {
		for dir in ["node_modules", "target", ".git", "__pycache__", ".cache"] {
			assert!(is_generated_dir(OsStr::new(dir)), "{dir} should be pruned");
		}
	}

	#[test]
	fn does_not_prune_names_that_merely_contain_a_match() {
		for dir in [
			"node_modulesAlbum",
			"targets",
			"distro",
			".gitignore",
			"builds",
		] {
			assert!(
				!is_generated_dir(OsStr::new(dir)),
				"{dir} should still be watched"
			);
		}
	}

	#[test]
	fn matches_case_sensitively() {
		assert!(!is_generated_dir(OsStr::new("Target")));
		assert!(!is_generated_dir(OsStr::new("Node_Modules")));
	}

	#[test]
	fn non_utf8_names_are_watched() {
		#[cfg(unix)]
		{
			use std::os::unix::ffi::OsStrExt;
			let invalid = OsStr::from_bytes(&[0x66, 0x6f, 0x80]);
			assert!(!is_generated_dir(invalid));
		}
	}
}
