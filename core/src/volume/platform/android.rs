//! Android volume detection (MOB-017)
//!
//! Android exposes the device's shared storage at `/storage/emulated/<user>` and each
//! removable card or USB drive at `/storage/<id>` (for example `/storage/1A2B-3C4D`).
//! The app reads them once the user grants "All files access"; their size is
//! readable either way through `statvfs`. Without these volumes the phone listed
//! no storage and its own device showed no files.

use std::path::{Path, PathBuf};

use tracing::debug;
use uuid::Uuid;

use crate::volume::{
	error::VolumeResult,
	types::{
		DiskType, FileSystem, MountType, Volume, VolumeDetectionConfig, VolumeFingerprint,
		VolumeType,
	},
};

/// Where Android mounts shared and removable storage.
const STORAGE_ROOT: &str = "/storage";

/// Detects the shared storage of the current user and every removable volume.
pub async fn detect_volumes(
	device_id: Uuid,
	_config: &VolumeDetectionConfig,
) -> VolumeResult<Vec<Volume>> {
	let volumes = detect_in(Path::new(STORAGE_ROOT), device_id, capacity);
	debug!(count = volumes.len(), "Android volume detection finished");
	Ok(volumes)
}

/// Lists the volumes under `root`, asking `capacity` for each one's
/// (total, available) bytes; a path `capacity` cannot measure is skipped.
fn detect_in(
	root: &Path,
	device_id: Uuid,
	capacity: impl Fn(&Path) -> Option<(u64, u64)>,
) -> Vec<Volume> {
	let mut volumes = Vec::new();

	// Shared storage of user 0, the one apps see as "Internal storage".
	let shared = root.join("emulated").join("0");
	if let Some((total, available)) = capacity(&shared) {
		volumes.push(volume(
			device_id,
			shared,
			"Internal storage".to_string(),
			VolumeType::Primary,
			MountType::System,
			total,
			available,
		));
	}

	// Removable volumes: every other entry except the emulated tree and `self`.
	let Ok(entries) = std::fs::read_dir(root) else {
		return volumes;
	};
	let mut removable: Vec<PathBuf> = entries
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.filter(|path| {
			path.file_name()
				.and_then(|name| name.to_str())
				.is_some_and(|name| name != "emulated" && name != "self")
		})
		.collect();
	removable.sort();
	for path in removable {
		if let Some((total, available)) = capacity(&path) {
			let name = path
				.file_name()
				.map(|name| name.to_string_lossy().into_owned())
				.unwrap_or_default();
			volumes.push(volume(
				device_id,
				path,
				name,
				VolumeType::External,
				MountType::External,
				total,
				available,
			));
		}
	}
	volumes
}

fn volume(
	device_id: Uuid,
	mount_point: PathBuf,
	name: String,
	volume_type: VolumeType,
	mount_type: MountType,
	total_capacity: u64,
	available_space: u64,
) -> Volume {
	let fingerprint = VolumeFingerprint::from_primary_volume(&mount_point, device_id);
	let now = chrono::Utc::now();
	Volume {
		id: Uuid::new_v5(&Uuid::NAMESPACE_OID, fingerprint.0.as_bytes()),
		fingerprint,
		device_id,
		name: name.clone(),
		library_id: None,
		is_tracked: false,
		mount_point: mount_point.clone(),
		mount_points: vec![mount_point],
		volume_type,
		mount_type,
		disk_type: DiskType::SSD,
		// Shared storage is a FUSE view over f2fs or ext4; cards are usually exFAT or FAT32.
		file_system: FileSystem::Other("android".to_string()),
		total_capacity,
		available_space,
		is_read_only: false,
		is_mounted: true,
		hardware_id: None,
		backend: None,
		cloud_identifier: None,
		cloud_config: None,
		apfs_container: None,
		container_volume_id: None,
		path_mappings: Vec::new(),
		is_user_visible: true,
		auto_track_eligible: false,
		read_speed_mbps: None,
		write_speed_mbps: None,
		created_at: now,
		updated_at: now,
		last_seen_at: now,
		total_files: None,
		total_directories: None,
		last_stats_update: None,
		display_name: Some(name),
		is_favorite: false,
		color: None,
		icon: None,
		error_message: None,
		supports_block_cloning: false,
	}
}

/// Total and available bytes of the filesystem holding `path`.
fn capacity(path: &Path) -> Option<(u64, u64)> {
	use std::os::unix::ffi::OsStrExt;

	let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
	let mut stats: libc::statvfs = unsafe { std::mem::zeroed() };
	if unsafe { libc::statvfs(path.as_ptr(), &mut stats) } != 0 {
		return None;
	}
	let block = stats.f_frsize as u64;
	let total = stats.f_blocks as u64 * block;
	(total > 0).then(|| (total, stats.f_bavail as u64 * block))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn shared_storage_and_cards_become_volumes() {
		let root = tempfile::tempdir().unwrap();
		std::fs::create_dir_all(root.path().join("emulated/0")).unwrap();
		std::fs::create_dir(root.path().join("self")).unwrap();
		std::fs::create_dir(root.path().join("1A2B-3C4D")).unwrap();
		std::fs::create_dir(root.path().join("unreadable")).unwrap();
		let device = Uuid::new_v4();

		let volumes = detect_in(root.path(), device, |path| {
			(!path.ends_with("unreadable")).then_some((64, 16))
		});

		let names: Vec<_> = volumes.iter().map(|v| v.name.as_str()).collect();
		assert_eq!(names, ["Internal storage", "1A2B-3C4D"]);
		assert_eq!(volumes[0].mount_point, root.path().join("emulated/0"));
		assert_eq!(volumes[0].volume_type, VolumeType::Primary);
		assert_eq!(volumes[1].volume_type, VolumeType::External);
		assert_eq!(
			(volumes[0].total_capacity, volumes[0].available_space),
			(64, 16)
		);
		// Stable ids: the same paths on the same device give the same volumes.
		let again = detect_in(root.path(), device, |_| Some((64, 16)));
		assert_eq!(again[0].id, volumes[0].id);
	}

	#[test]
	fn the_real_filesystem_reports_a_capacity() {
		let (total, available) = capacity(Path::new("/")).unwrap();
		assert!(total > 0 && available <= total);
	}
}
