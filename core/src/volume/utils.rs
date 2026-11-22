//! Shared utilities for volume detection across platforms

use crate::{
	domain::volume::WingDriveVolumeId,
	volume::{
		error::{VolumeError, VolumeResult},
		types::FileSystem,
	},
};
use std::path::Path;
use tracing::warn;
use uuid::Uuid;

/// Parse size strings from df output (e.g., "1.5G", "931Gi", "1024K")
pub fn parse_size_string(size_str: &str) -> VolumeResult<u64> {
	if size_str == "-" {
		return Ok(0);
	}

	// Skip invalid size strings that don't look like numbers
	if size_str.is_empty() || size_str.chars().all(char::is_alphabetic) {
		return Ok(0);
	}

	let size_str = size_str.replace(",", ""); // Remove commas
	let (number_part, unit) = if let Some(pos) = size_str.find(char::is_alphabetic) {
		(&size_str[..pos], &size_str[pos..])
	} else {
		(size_str.as_str(), "")
	};

	let number: f64 = number_part
		.parse()
		.map_err(|_| VolumeError::InvalidData(format!("Invalid size: {}", size_str)))?;

	let multiplier = match unit.to_uppercase().as_str() {
		"" | "B" => 1,
		"K" | "KB" | "KI" => 1024,
		"M" | "MB" | "MI" => 1024 * 1024,
		"G" | "GB" | "GI" => 1024 * 1024 * 1024,
		"T" | "TB" | "TI" => 1024u64.pow(4),
		"P" | "PB" | "PI" => 1024u64.pow(5),
		_ => {
			warn!("Unknown size unit: {}", unit);
			1
		}
	};

	Ok((number * multiplier as f64) as u64)
}

/// Check if a filesystem should be considered a system filesystem
pub fn is_system_filesystem(filesystem: &str) -> bool {
	matches!(
		filesystem,
		"/" | "/dev" | "/proc" | "/sys" | "/tmp" | "/var/tmp"
	)
}

/// Check if a filesystem is virtual (not backed by physical storage)
pub fn is_virtual_filesystem(filesystem: &str) -> bool {
	let fs_lower = filesystem.to_lowercase();
	matches!(
		fs_lower.as_str(),
		"devfs"
			| "sysfs"
			| "proc"
			| "tmpfs"
			| "ramfs"
			| "devtmpfs"
			| "overlay"
			| "overlayfs"
			| "fuse"
			| "fusectl"
			| "cgroup"
			| "cgroup2"
			| "squashfs"
			| "efivarfs"
			| "pstore"
			| "bpf"
			| "debugfs"
			| "tracefs"
			| "configfs"
			| "securityfs"
			| "autofs"
			| "binfmt_misc"
			| "mqueue"
			| "hugetlbfs"
			| "rpc_pipefs"
			| "nsfs"
			| "selinuxfs"
			| "pipefs"
			| "sockfs"
	) || fs_lower.starts_with("map ")
		|| fs_lower.contains("auto_")
}

/// Check if a Linux mount point is a known OS/system path that should be
/// hidden from the user-visible volume list.
///
/// Covers the standard Linux filesystem hierarchy plus TrueNAS Scale's
/// additional split-root datasets (it uses separate ZFS datasets for
/// `/usr`, `/var`, `/etc`, etc. to support atomic updates).
///
/// `/` and `/home` are deliberately absent. They are the Linux equivalents of
/// Macintosh HD, which macOS classifies as `VolumeType::Primary` and shows;
/// the Linux classifier agrees (`/` => Primary, `/home` => UserData). Hiding
/// them left a desktop install with no way to reach the user's own files.
#[cfg(target_os = "linux")]
pub fn is_system_mount_point(mount_point: &Path) -> bool {
	let path_str = mount_point.to_string_lossy();

	// Exact matches for system paths
	matches!(
		path_str.as_ref(),
		"/usr" | "/var"
			| "/etc"
			| "/opt"
			| "/srv"
			| "/root"
			| "/boot"
			| "/run"
			| "/dev"
			| "/proc"
			| "/sys"
			| "/tmp"
			| "/audit"
			| "/data"
			| "/conf"
			| "/mnt"
			| "/lost+found"
	) ||
	// System subpaths
	path_str.starts_with("/boot/")
		|| path_str.starts_with("/sys/")
		|| path_str.starts_with("/proc/")
		|| path_str.starts_with("/dev/")
		|| path_str.starts_with("/run/")
		|| path_str.starts_with("/var/log")
		|| path_str.starts_with("/var/db/")
		|| path_str.starts_with("/var/lib/systemd")
		|| path_str.starts_with("/var/local/")
		|| path_str.starts_with("/var/cache/")
}

/// Check if a mount point looks like a nested app/container volume that
/// shouldn't be exposed as a user-facing volume.
///
/// Catches Docker/Kubernetes layer mounts, snap packages, ZFS snapshot
/// directories, and TrueNAS Scale's iX Apps (each app creates many
/// nested ZFS datasets under `ix-applications/` — and the root of that
/// hierarchy itself should also be hidden).
#[cfg(target_os = "linux")]
pub fn is_nested_app_mount(mount_point: &Path) -> bool {
	let path_str = mount_point.to_string_lossy();

	// Match root directories as well as anything under them
	let is_under = |needle: &str| -> bool {
		path_str.contains(&format!("/{needle}/")) || path_str.ends_with(&format!("/{needle}"))
	};

	is_under(".ix-apps")
		|| is_under("ix-applications")
		|| path_str.contains("/docker/overlay2/")
		|| path_str.contains("/containerd/")
		|| path_str.contains("/kubelet/")
		|| path_str.contains("/snap/")
		|| path_str.contains("/.snapshots/")
		|| path_str.contains("/.zfs/snapshot/")
}

/// Re-evaluate whether a mount path should be hidden from the user based on
/// current platform visibility rules. Shared between the volume list query
/// and the library statistics calculation so both consistently hide system
/// mounts and nested app/container volumes even when the tracked DB row
/// predates these filters (and so has a stale `is_user_visible = true`).
#[cfg(target_os = "linux")]
pub fn should_hide_by_mount_path(mount_point: &Path) -> bool {
	is_system_mount_point(mount_point) || is_nested_app_mount(mount_point)
}

#[cfg(not(target_os = "linux"))]
pub fn should_hide_by_mount_path(_mount_point: &Path) -> bool {
	false
}

/// Parse filesystem type from string to FileSystem enum
pub fn parse_filesystem_type(fs_type: &str) -> FileSystem {
	match fs_type.to_lowercase().as_str() {
		"apfs" => FileSystem::APFS,
		"btrfs" => FileSystem::Btrfs,
		"zfs" => FileSystem::ZFS,
		"refs" => FileSystem::ReFS,
		"ntfs" => FileSystem::NTFS,
		"ext2" | "ext3" | "ext4" => FileSystem::Ext4,
		"xfs" => FileSystem::Other("XFS".to_string()),
		"fat32" | "vfat" => FileSystem::FAT32,
		"exfat" => FileSystem::ExFAT,
		"hfs+" | "hfsplus" => FileSystem::Other("HFS+".to_string()),
		_ => FileSystem::Other(fs_type.to_string()),
	}
}

/// Read or create .wingdrive-volume-id file on external volumes (async version)
/// This file provides persistent identification for removable drives
/// Returns the UUID from the file, or None if the volume is read-only
pub async fn read_or_create_dotfile(
	mount_point: &Path,
	device_id: Uuid,
	library_id: Option<Uuid>,
) -> Option<Uuid> {
	let mount_point = mount_point.to_owned();
	tokio::task::spawn_blocking(move || {
		read_or_create_dotfile_sync(&mount_point, device_id, library_id)
	})
	.await
	.ok()
	.flatten()
}

/// Read or create .wingdrive-volume-id file on external volumes (sync version)
/// This file provides persistent identification for removable drives
/// Returns the UUID from the file, or None if the volume is read-only
pub fn read_or_create_dotfile_sync(
	mount_point: &Path,
	device_id: Uuid,
	library_id: Option<Uuid>,
) -> Option<Uuid> {
	let id_file_path = crate::branding::volume_id_file(mount_point);

	let read_id = || -> std::io::Result<Uuid> {
		let file = std::fs::File::open(&id_file_path)?;
		let marker: WingDriveVolumeId = serde_json::from_reader(file)?;
		Ok(marker.id)
	};
	match read_id() {
		Ok(id) => return Some(id),
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
		Err(_) => return None,
	}
	let wingdrive_id = WingDriveVolumeId {
		id: Uuid::new_v4(),
		created: chrono::Utc::now(),
		device_name: None,
		volume_name: mount_point
			.file_name()
			.map(|n| n.to_string_lossy().to_string())
			.unwrap_or_else(|| "Unknown".to_string()),
		device_id,
		library_id: library_id.unwrap_or(Uuid::nil()),
	};

	let mut staged = tempfile::NamedTempFile::new_in(mount_point).ok()?;
	serde_json::to_writer_pretty(staged.as_file_mut(), &wingdrive_id).ok()?;
	staged.as_file().sync_all().ok()?;
	match staged.persist_noclobber(&id_file_path) {
		Ok(_) => Some(wingdrive_id.id),
		Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => read_id().ok(),
		Err(_) => None,
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn simultaneous_hosts_read_the_same_portable_marker() {
		let volume = tempfile::tempdir().unwrap();
		let barrier = std::sync::Barrier::new(4);
		let ids = std::thread::scope(|scope| {
			let workers: Vec<_> = (0..4)
				.map(|_| {
					scope.spawn(|| {
						barrier.wait();
						super::read_or_create_dotfile_sync(
							volume.path(),
							uuid::Uuid::new_v4(),
							None,
						)
						.unwrap()
					})
				})
				.collect();
			workers
				.into_iter()
				.map(|worker| worker.join().unwrap())
				.collect::<Vec<_>>()
		});
		assert!(ids.iter().all(|id| id == &ids[0]));
	}

	#[test]
	fn damaged_portable_marker_survives_detection() {
		let volume = tempfile::tempdir().unwrap();
		let marker = crate::branding::volume_id_file(volume.path());
		std::fs::write(&marker, b"broken").unwrap();
		assert!(
			super::read_or_create_dotfile_sync(volume.path(), uuid::Uuid::new_v4(), None).is_none()
		);
		assert_eq!(std::fs::read(marker).unwrap(), b"broken");
	}

	use super::*;

	#[test]
	fn test_parse_size_string() {
		assert_eq!(parse_size_string("1024").unwrap(), 1024);
		assert_eq!(parse_size_string("1K").unwrap(), 1024);
		assert_eq!(parse_size_string("1M").unwrap(), 1024 * 1024);
		assert_eq!(parse_size_string("1G").unwrap(), 1024 * 1024 * 1024);
		assert_eq!(
			parse_size_string("1.5G").unwrap(),
			(1.5 * 1024.0 * 1024.0 * 1024.0) as u64
		);
		assert_eq!(parse_size_string("-").unwrap(), 0);
	}

	#[test]
	fn test_filesystem_detection() {
		assert!(is_virtual_filesystem("tmpfs"));
		assert!(is_virtual_filesystem("proc"));
		assert!(!is_virtual_filesystem("ext4"));

		assert!(is_system_filesystem("/"));
		assert!(is_system_filesystem("/proc"));
		assert!(!is_system_filesystem("/home"));
	}

	#[test]
	fn test_parse_filesystem_type() {
		assert!(matches!(parse_filesystem_type("apfs"), FileSystem::APFS));
		assert!(matches!(parse_filesystem_type("btrfs"), FileSystem::Btrfs));
		assert!(matches!(parse_filesystem_type("ext4"), FileSystem::Ext4));
		assert!(matches!(
			parse_filesystem_type("unknown"),
			FileSystem::Other(_)
		));
	}

	#[cfg(target_os = "linux")]
	#[test]
	fn test_should_hide_by_mount_path_linux() {
		// System mounts — always hidden
		assert!(should_hide_by_mount_path(Path::new("/usr")));
		assert!(should_hide_by_mount_path(Path::new("/var")));
		assert!(should_hide_by_mount_path(Path::new("/etc")));
		assert!(should_hide_by_mount_path(Path::new("/boot/grub")));
		assert!(should_hide_by_mount_path(Path::new("/var/log/journal")));
		assert!(should_hide_by_mount_path(Path::new(
			"/var/db/system/netdata"
		)));
		assert!(should_hide_by_mount_path(Path::new("/sys/firmware/efi")));

		// TrueNAS Scale app-managed datasets
		assert!(should_hide_by_mount_path(Path::new("/mnt/.ix-apps")));
		assert!(should_hide_by_mount_path(Path::new("/mnt/.ix-apps/docker")));
		assert!(should_hide_by_mount_path(Path::new(
			"/mnt/pool/ix-applications"
		)));
		assert!(should_hide_by_mount_path(Path::new(
			"/mnt/pool/ix-applications/releases/plex/volumes/ix_volumes/ix-plex_data"
		)));

		// The desktop's own drives — the whole point of a file browser
		assert!(!should_hide_by_mount_path(Path::new("/")));
		assert!(!should_hide_by_mount_path(Path::new("/home")));

		// User data — not hidden
		assert!(!should_hide_by_mount_path(Path::new("/mnt/pool")));
		assert!(!should_hide_by_mount_path(Path::new("/mnt/pool/footage")));
		assert!(!should_hide_by_mount_path(Path::new(
			"/mnt/pool/calvin-nas"
		)));
	}
}
