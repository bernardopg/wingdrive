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
//! On Linux the path's own filesystem comes from `statfs`, because the disk list
//! skips virtual mounts such as tmpfs and would attribute `/tmp` to the root
//! filesystem. Other platforms resolve the path against the mount table.

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

		#[cfg(target_os = "linux")]
		if let Some(is_cow) = linux_statfs_is_cow(&target) {
			return Some(is_cow);
		}

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

/// Asks the kernel which filesystem holds `path` (or its parent, when the path
/// itself is gone). Returns `None` when `statfs` fails.
#[cfg(target_os = "linux")]
fn linux_statfs_is_cow(path: &Path) -> Option<bool> {
	use std::{ffi::CString, os::unix::ffi::OsStrExt};

	// Magic numbers from linux/magic.h and the ZFS/bcachefs sources
	const BTRFS_SUPER_MAGIC: i64 = 0x9123_683E;
	const ZFS_SUPER_MAGIC: i64 = 0x2FC1_2FC1;
	const BCACHEFS_SUPER_MAGIC: i64 = 0xCA45_1A4E;

	let probe = path
		.ancestors()
		.find(|p| p.exists())
		.unwrap_or(Path::new("/"));
	let c_path = CString::new(probe.as_os_str().as_bytes()).ok()?;
	let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
	// SAFETY: `c_path` is a valid NUL-terminated string and `stat` is a valid
	// out-pointer for one `statfs` struct.
	if unsafe { libc::statfs(c_path.as_ptr(), stat.as_mut_ptr()) } != 0 {
		return None;
	}
	// SAFETY: `statfs` returned 0, so it filled `stat`.
	#[allow(clippy::unnecessary_cast)]
	let f_type = unsafe { stat.assume_init() }.f_type as i64;
	let is_cow = matches!(
		f_type,
		BTRFS_SUPER_MAGIC | ZFS_SUPER_MAGIC | BCACHEFS_SUPER_MAGIC
	);
	debug!(path = %path.display(), f_type, is_cow, "Resolved filesystem for secure delete");
	Some(is_cow)
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

	#[cfg(target_os = "linux")]
	#[test]
	fn tmpfs_is_not_reported_as_cow() {
		// /dev/shm is tmpfs on every mainstream distro
		assert_eq!(linux_statfs_is_cow(Path::new("/dev/shm")), Some(false));
	}

	#[test]
	fn overwrite_in_place_filesystems_are_not_cow() {
		for name in ["ext4", "xfs", "ntfs", "fat32"] {
			assert!(!filesystem_is_cow(name));
		}
	}
}
