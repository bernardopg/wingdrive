//! # Network locations and devices through GIO
//!
//! Phones (MTP), cameras, SMB shares and SFTP servers are reached on Linux
//! desktops through gvfs, which exposes every mount as a FUSE folder under
//! `$XDG_RUNTIME_DIR/gvfs`. WingDrive browses those folders like any other
//! local path, so this module only lists them, lists devices that can be
//! mounted, and mounts or unmounts through the `gio` tool.
//!
//! `gio mount` asks for credentials on stdin. A password, when given, is the
//! only line written; the user name and domain belong in the URI
//! (`smb://DOMAIN;user@host/share`), so the prompt order never matters.
//!
//! ## Example
//! ```rust,ignore
//! let mounted = gvfs::mount("sftp://me@server/".into(), None).await?;
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::Serialize;

/// A mounted gvfs location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NetworkMount {
	/// Readable name, such as `me on server (SFTP)`.
	pub name: String,
	/// FUSE folder that WingDrive browses.
	pub path: String,
	/// Scheme of the mount, such as `sftp`, `smb-share` or `mtp`.
	pub kind: String,
}

/// A device or volume gvfs can mount but has not mounted yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MountableVolume {
	pub name: String,
	/// URI passed to `gio mount`, such as `mtp://Google_Pixel_123/`.
	pub uri: String,
}

fn gvfs_root() -> Option<PathBuf> {
	std::env::var_os("XDG_RUNTIME_DIR")
		.map(PathBuf::from)
		.or_else(|| {
			Some(PathBuf::from(format!("/run/user/{}", unsafe {
				libc::getuid()
			})))
		})
		.map(|dir| dir.join("gvfs"))
}

fn gio() -> Command {
	let mut command = Command::new("gio");
	file_opening_linux::use_host_environment(&mut command);
	command
}

/// Turns a FUSE folder name such as `sftp:host=server,user=me` into a label.
pub fn describe(folder: &str) -> NetworkMount {
	let (kind, rest) = folder.split_once(':').unwrap_or((folder, ""));
	let fields: Vec<(&str, &str)> = rest
		.split(',')
		.filter_map(|kv| kv.split_once('='))
		.collect();
	let get = |key: &str| {
		fields
			.iter()
			.find(|(k, _)| *k == key)
			.map(|(_, v)| percent_decode(v))
	};
	let host = get("host").unwrap_or_default();
	let place = match (get("share"), get("user")) {
		(Some(share), _) => format!("{share} on {host}"),
		(None, Some(user)) => format!("{user} on {host}"),
		(None, None) => host,
	};
	let label = match kind {
		"sftp" => "SFTP",
		"smb-share" => "SMB",
		"ftp" | "ftps" => "FTP",
		"dav" | "davs" => "WebDAV",
		"mtp" => "Phone",
		"gphoto2" => "Camera",
		"afc" => "iPhone",
		"nfs" => "NFS",
		other => other,
	};
	let name = if place.is_empty() {
		label.to_string()
	} else {
		format!("{place} ({label})")
	};
	NetworkMount {
		name,
		path: String::new(),
		kind: kind.to_string(),
	}
}

fn percent_decode(text: &str) -> String {
	let bytes = text.as_bytes();
	let mut out = Vec::with_capacity(bytes.len());
	let mut i = 0;
	while i < bytes.len() {
		if bytes[i] == b'%' {
			if let Some(byte) = std::str::from_utf8(&bytes[i + 1..(i + 3).min(bytes.len())])
				.ok()
				.and_then(|hex| u8::from_str_radix(hex, 16).ok())
			{
				out.push(byte);
				i += 3;
				continue;
			}
		}
		out.push(bytes[i]);
		i += 1;
	}
	String::from_utf8_lossy(&out).into_owned()
}

fn list_in(root: &Path) -> Vec<NetworkMount> {
	let Ok(entries) = std::fs::read_dir(root) else {
		return Vec::new();
	};
	let mut mounts: Vec<NetworkMount> = entries
		.flatten()
		.map(|entry| {
			let folder = entry.file_name().to_string_lossy().into_owned();
			NetworkMount {
				path: entry.path().to_string_lossy().into_owned(),
				..describe(&folder)
			}
		})
		.collect();
	mounts.sort_by_key(|m| m.name.to_lowercase());
	mounts
}

/// Parses `gio mount -li` for volumes that are not mounted and have a URI.
pub fn parse_mountable(listing: &str) -> Vec<MountableVolume> {
	struct Block {
		indent: usize,
		name: String,
		uri: Option<String>,
		can_mount: bool,
		mounted: bool,
	}
	fn flush(block: Option<Block>, out: &mut Vec<MountableVolume>) {
		if let Some(Block {
			name,
			uri: Some(uri),
			can_mount: true,
			mounted: false,
			..
		}) = block
		{
			out.push(MountableVolume { name, uri });
		}
	}
	let mut volumes = Vec::new();
	let mut current: Option<Block> = None;
	for line in listing.lines() {
		let trimmed = line.trim_start();
		let indent = line.len() - trimmed.len();
		let inside = current.as_ref().is_some_and(|b| indent > b.indent);
		if let Some(rest) = trimmed.strip_prefix("Volume(") {
			flush(current.take(), &mut volumes);
			let name = rest
				.split_once("): ")
				.map(|(_, n)| n.trim().to_string())
				.unwrap_or_default();
			current = Some(Block {
				indent,
				name,
				uri: None,
				can_mount: false,
				mounted: false,
			});
		} else if !inside {
			flush(current.take(), &mut volumes);
		} else if let Some(block) = current.as_mut() {
			if trimmed.starts_with("Mount(") {
				block.mounted = true;
			} else if let Some(uri) = trimmed.strip_prefix("activation_root=") {
				block.uri = Some(uri.trim().to_string());
			} else if trimmed == "can_mount=1" {
				block.can_mount = true;
			}
		}
	}
	flush(current.take(), &mut volumes);
	volumes
}

/// Mounted network locations and devices.
#[tauri::command]
pub fn list_network_mounts() -> Vec<NetworkMount> {
	gvfs_root().map(|root| list_in(&root)).unwrap_or_default()
}

/// Devices such as phones and cameras that gvfs can mount.
#[tauri::command]
pub async fn list_mountable_volumes() -> Result<Vec<MountableVolume>, String> {
	let output = tokio::task::spawn_blocking(|| gio().args(["mount", "-li"]).output())
		.await
		.map_err(|e| e.to_string())?
		.map_err(|e| format!("gio is not available: {e}"))?;
	Ok(parse_mountable(&String::from_utf8_lossy(&output.stdout)))
}

/// Mounts `uri` and returns the new mount's folder.
///
/// Fails after 90 seconds, which covers a slow server but stops a prompt
/// gio could never get an answer to (an unknown host key, a missing user).
#[tauri::command]
pub async fn mount_network_location(
	uri: String,
	password: Option<String>,
) -> Result<NetworkMount, String> {
	let uri = uri.trim().to_string();
	if !uri.contains("://") {
		return Err("Enter an address such as sftp://user@server/ or smb://server/share".into());
	}
	let before = list_network_mounts();
	tokio::task::spawn_blocking(move || run_mount(&uri, password.as_deref()))
		.await
		.map_err(|e| e.to_string())??;
	let after = list_network_mounts();
	after
		.iter()
		.find(|m| !before.iter().any(|b| b.path == m.path))
		.or_else(|| after.first())
		.cloned()
		.ok_or_else(|| "Mounted, but the location did not appear under gvfs".to_string())
}

fn run_mount(uri: &str, password: Option<&str>) -> Result<(), String> {
	let mut child = gio()
		.args(["mount", uri])
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.map_err(|e| format!("gio is not available: {e}"))?;
	if let Some(mut stdin) = child.stdin.take() {
		if let Some(password) = password {
			let _ = writeln!(stdin, "{password}");
		}
		// Closing stdin makes any further prompt fail instead of waiting.
	}
	let deadline = std::time::Instant::now() + Duration::from_secs(90);
	loop {
		if let Some(status) = child.try_wait().map_err(|e| e.to_string())? {
			let output = child.wait_with_output().map_err(|e| e.to_string())?;
			let stderr = String::from_utf8_lossy(&output.stderr);
			// "Already mounted" counts as success for the caller.
			if status.success() || stderr.contains("already mounted") {
				return Ok(());
			}
			let message = stderr.lines().last().unwrap_or("").trim();
			return Err(if message.is_empty() {
				format!("Could not mount {uri}")
			} else {
				message.trim_start_matches("gio: ").to_string()
			});
		}
		if std::time::Instant::now() > deadline {
			let _ = child.kill();
			let _ = child.wait();
			return Err(format!("Timed out connecting to {uri}"));
		}
		std::thread::sleep(Duration::from_millis(100));
	}
}

/// Unmounts the mount whose FUSE folder is `path`.
#[tauri::command]
pub async fn unmount_network_location(path: String) -> Result<(), String> {
	let root = gvfs_root().ok_or("No gvfs folder")?;
	let path = PathBuf::from(path);
	if path.parent() != Some(root.as_path()) {
		return Err("Not a network location".into());
	}
	let output =
		tokio::task::spawn_blocking(move || gio().arg("mount").arg("-u").arg(&path).output())
			.await
			.map_err(|e| e.to_string())?
			.map_err(|e| e.to_string())?;
	if output.status.success() {
		Ok(())
	} else {
		Err(String::from_utf8_lossy(&output.stderr)
			.trim()
			.trim_start_matches("gio: ")
			.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn folder_names_become_labels() {
		assert_eq!(
			describe("sftp:host=server,user=me").name,
			"me on server (SFTP)"
		);
		assert_eq!(
			describe("smb-share:server=nas,share=media,host=nas").name,
			"media on nas (SMB)"
		);
		assert_eq!(
			describe("mtp:host=Google_Pixel_7_ABC").name,
			"Google_Pixel_7_ABC (Phone)"
		);
		assert_eq!(
			describe("dav:host=cloud,ssl=true,user=a%40b.c").name,
			"a@b.c on cloud (WebDAV)"
		);
		assert_eq!(describe("sftp:host=server,user=me").kind, "sftp");
	}

	#[test]
	fn lists_fuse_folders() {
		let root = tempfile::tempdir().unwrap();
		std::fs::create_dir(root.path().join("sftp:host=b,user=x")).unwrap();
		std::fs::create_dir(root.path().join("mtp:host=Phone")).unwrap();
		let names: Vec<_> = list_in(root.path()).into_iter().map(|m| m.name).collect();
		assert_eq!(names, vec!["Phone (Phone)", "x on b (SFTP)"]);
		assert!(list_in(&root.path().join("missing")).is_empty());
	}

	#[test]
	fn only_unmounted_volumes_with_uris_are_mountable() {
		let listing = "\
Drive(0): WD Green
  Type: GProxyDrive (GProxyVolumeMonitorUDisks2)
  Volume(0): 999 GB Volume
    Type: GProxyVolume (GProxyVolumeMonitorUDisks2)
    can_mount=1
Volume(0): Pixel 7
  Type: GProxyVolume (GProxyVolumeMonitorMTP)
  activation_root=mtp://Google_Pixel_7_ABC/
  can_mount=1
Volume(1): Camera
  Type: GProxyVolume (GProxyVolumeMonitorGPhoto2)
  activation_root=gphoto2://Canon/
  can_mount=1
  Mount(0): Camera -> gphoto2://Canon/
    Type: GProxyMount (GProxyVolumeMonitorGPhoto2)
";
		assert_eq!(
			parse_mountable(listing),
			vec![MountableVolume {
				name: "Pixel 7".into(),
				uri: "mtp://Google_Pixel_7_ABC/".into()
			}]
		);
	}
}
