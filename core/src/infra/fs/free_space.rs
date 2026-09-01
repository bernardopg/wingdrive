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
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//! use sd_core::infra::fs::free_space::ensure_headroom;
//!
//! ensure_headroom(Path::new("/library"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

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
	available_bytes_on_mounts(
		&target,
		disks
			.list()
			.iter()
			.map(|disk| (disk.mount_point(), disk.available_space())),
	)
}

fn available_bytes_on_mounts<'a>(
	target: &Path,
	mounts: impl IntoIterator<Item = (&'a Path, u64)>,
) -> Option<u64> {
	let mut best: Option<(usize, u64)> = None;

	for (mount_point, available) in mounts {
		if !target.starts_with(mount_point) {
			continue;
		}

		let depth = mount_point.components().count();
		if best.is_none_or(|(best_depth, _)| depth > best_depth) {
			best = Some((depth, available));
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
	let available = available_bytes(path);
	if available.is_none() {
		tracing::debug!(
			path = %path.display(),
			"No mount point matched; skipping free space preflight"
		);
	}

	ensure_available_space(available, required_bytes)
}

fn ensure_available_space(
	available: Option<u64>,
	required_bytes: u64,
) -> Result<(), InsufficientSpace> {
	let Some(available) = available else {
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
	fn selects_the_deepest_matching_mount() {
		let target = Path::new("/workspace/library/files");
		assert_eq!(
			available_bytes_on_mounts(
				target,
				[
					(Path::new("/"), 1_000),
					(Path::new("/workspace"), 2_000),
					(Path::new("/workspace/library"), 3_000),
				],
			),
			Some(3_000)
		);
	}

	#[test]
	fn does_not_match_a_shared_path_prefix() {
		assert_eq!(
			available_bytes_on_mounts(
				Path::new("/workspace2/files"),
				[(Path::new("/workspace"), 1_000)],
			),
			None
		);
	}

	#[test]
	fn permits_an_unverifiable_mount() {
		assert!(ensure_available_space(None, u64::MAX).is_ok());
	}

	#[test]
	fn accepts_a_write_that_preserves_the_reserve() {
		assert!(ensure_available_space(Some(RESERVE_BYTES + 1_024), 1_024).is_ok());
	}

	#[test]
	fn rejects_a_write_that_would_consume_the_reserve() {
		let error = ensure_available_space(Some(RESERVE_BYTES + 1_023), 1_024).unwrap_err();
		assert_eq!(error.required_bytes, 1_024);
		assert_eq!(error.available_bytes, RESERVE_BYTES + 1_023);
	}
}
