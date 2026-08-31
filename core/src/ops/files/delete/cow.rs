//! # Copy-on-Write Filesystem Detection
//!
//! Secure deletion works by overwriting a file's bytes in place before unlinking it.
//! That assumption breaks on copy-on-write filesystems: btrfs, ZFS, APFS, ReFS and
//! bcachefs never reuse the original extents, so every "overwrite" pass allocates
//! fresh blocks and leaves the original data intact until the extents are reclaimed.
//!
//! The result is worse than useless. The secret survives, and a three-pass overwrite
//! of an N-byte file writes 3N bytes of new extents plus checksum metadata before
//! freeing anything. On a nearly-full btrfs volume that is enough to exhaust the
//! metadata block groups and force the filesystem read-only, which on a root
//! filesystem means an unbootable machine.
//!
//! Detection runs against the mount table rather than a syscall so the same code
//! path works on every platform the daemon targets.

use std::path::Path;
use tracing::debug;

/// Filesystems that redirect writes to new extents instead of overwriting in place.
const COW_FILESYSTEMS: &[&str] = &["btrfs", "zfs", "apfs", "refs", "bcachefs"];

fn filesystem_is_cow(name: &str) -> bool {
	let name = name.to_lowercase();
	COW_FILESYSTEMS.iter().any(|known| name.contains(known))
}

/// Cached mount information for checking every entry before a directory delete.
pub(super) struct CowFilesystemDetector {
	disks: sysinfo::Disks,
}

impl CowFilesystemDetector {
	pub(super) fn new() -> Self {
		Self {
			disks: sysinfo::Disks::new_with_refreshed_list(),
		}
	}

	/// Reports whether `path` lives on a copy-on-write filesystem.
	///
	/// Resolves the longest mount point that prefixes the path, which makes nested
	/// mounts answer correctly. An unresolvable path returns `None` so secure delete
	/// can fail closed instead of claiming an unknown filesystem supports overwrites.
	pub(super) fn is_cow_filesystem(&self, path: &Path) -> Option<bool> {
		let target = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
		let mut best_match: Option<(usize, String)> = None;

		for disk in self.disks.list() {
			let mount_point = disk.mount_point();
			if !target.starts_with(mount_point) {
				continue;
			}

			let depth = mount_point.components().count();
			let fs_name = disk.file_system().to_string_lossy().to_lowercase();

			match &best_match {
				Some((best_depth, _)) if *best_depth >= depth => {}
				_ => best_match = Some((depth, fs_name)),
			}
		}

		match best_match {
			Some((_, fs_name)) => {
				let is_cow = filesystem_is_cow(&fs_name);
				debug!(
					path = %target.display(),
					filesystem = %fs_name,
					is_cow,
					"Resolved filesystem for secure delete"
				);
				Some(is_cow)
			}
			None => {
				debug!(
					path = %target.display(),
					"No mount point matched for secure delete"
				);
				None
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn known_cow_names_are_recognized() {
		for name in COW_FILESYSTEMS {
			assert!(filesystem_is_cow(name));
		}
		assert!(filesystem_is_cow("BTRFS"));
	}

	#[test]
	fn overwrite_in_place_filesystems_are_not_cow() {
		for name in ["ext4", "xfs", "ntfs", "fat32"] {
			assert!(!filesystem_is_cow(name));
		}
	}
}
