//! # Free Space Preflight
//!
//! Jobs that write large amounts of data fail badly when the target volume runs
//! out of space: a copy leaves truncated files behind, and filling a volume that
//! hosts the operating system can make the machine unusable before the job ever
//! reports an error. Checking up front turns that into a refusal.
//!
//! The reserve exists because filesystems degrade before they reach zero. ext4
//! fragments badly in the last few percent, btrfs can fail to allocate metadata
//! block groups while raw bytes remain, and a full root volume takes down
//! logging and session state with it.

use std::path::Path;

/// Bytes deliberately left unused on the destination volume.
const RESERVE_BYTES: u64 = 256 * 1024 * 1024;

/// Free space on the volume containing `path`, or `None` when no mount matches.
///
/// Resolves the longest matching mount point so a nested mount answers for
/// itself rather than for its parent. An unresolvable path returns `None` and
/// callers treat that as "cannot verify" rather than "no space".
pub fn available_bytes(path: &Path) -> Option<u64> {
	let target = path.canonicalize().unwrap_or_else(|_| {
		// A destination that does not exist yet still lives on an existing parent.
		path.ancestors()
			.find(|ancestor| ancestor.exists())
			.map(Path::to_path_buf)
			.unwrap_or_else(|| path.to_path_buf())
	});

	let disks = sysinfo::Disks::new_with_refreshed_list();
	let mut best: Option<(usize, u64)> = None;

	for disk in disks.list() {
		let mount_point = disk.mount_point();
		if !target.starts_with(mount_point) {
			continue;
		}

		let depth = mount_point.components().count();
		if best.is_none_or(|(best_depth, _)| depth > best_depth) {
			best = Some((depth, disk.available_space()));
		}
	}

	best.map(|(_, available)| available)
}

/// Rejects a write of `required_bytes` when the destination volume cannot hold it.
///
/// Returns `Ok(())` when the volume has room, or when free space cannot be
/// determined: refusing on an unknown mount would break perfectly valid writes to
/// network and virtual filesystems that report no capacity.
pub fn ensure_space_for(path: &Path, required_bytes: u64) -> Result<(), InsufficientSpace> {
	let Some(available) = available_bytes(path) else {
		tracing::debug!(
			path = %path.display(),
			"No mount point matched; skipping free space preflight"
		);
		return Ok(());
	};

	let needed = required_bytes.saturating_add(RESERVE_BYTES);
	if available >= needed {
		return Ok(());
	}

	Err(InsufficientSpace {
		required_bytes,
		available_bytes: available,
		reserve_bytes: RESERVE_BYTES,
	})
}

/// Rejects starting derived-data generation on a volume that is already full.
///
/// Thumbnails and proxies have no reliable size estimate up front, so the guard
/// is a headroom check instead: refuse when the volume holding the library has
/// less than the reserve free, rather than discovering it mid-run after writing
/// thousands of files.
pub fn ensure_headroom(path: &Path) -> Result<(), InsufficientSpace> {
	ensure_space_for(path, 0)
}

/// The destination volume cannot hold the requested write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsufficientSpace {
	pub required_bytes: u64,
	pub available_bytes: u64,
	pub reserve_bytes: u64,
}

impl std::fmt::Display for InsufficientSpace {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"not enough free space: {} required plus {} reserved, but only {} available",
			crate::infra::job::progress::format_bytes(self.required_bytes),
			crate::infra::job::progress::format_bytes(self.reserve_bytes),
			crate::infra::job::progress::format_bytes(self.available_bytes)
		)
	}
}

impl std::error::Error for InsufficientSpace {}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reports_available_space_for_an_existing_path() {
		let temp = tempfile::tempdir().unwrap();
		assert!(available_bytes(temp.path()).is_some());
	}

	#[test]
	fn resolves_through_a_destination_that_does_not_exist_yet() {
		let temp = tempfile::tempdir().unwrap();
		let missing = temp.path().join("not-created-yet/nested");
		assert_eq!(available_bytes(&missing), available_bytes(temp.path()));
	}

	#[test]
	fn accepts_a_write_that_fits() {
		let temp = tempfile::tempdir().unwrap();
		assert!(ensure_space_for(temp.path(), 1024).is_ok());
	}

	#[test]
	fn rejects_a_write_larger_than_the_volume() {
		let temp = tempfile::tempdir().unwrap();
		let error = ensure_space_for(temp.path(), u64::MAX / 2).unwrap_err();
		assert!(error.to_string().contains("not enough free space"));
	}

	#[test]
	fn reserve_is_kept_free() {
		let temp = tempfile::tempdir().unwrap();
		let available = available_bytes(temp.path()).unwrap();
		// Asking for everything must fail, because the reserve is added on top.
		assert!(ensure_space_for(temp.path(), available).is_err());
	}
}
