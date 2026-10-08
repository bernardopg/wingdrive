//! # Safe Desktop Undo
//!
//! Linux undo checks the inode and uses an atomic no-replace rename so an
//! intervening filesystem change cannot cause an existing file to be overwritten.

#[cfg(target_os = "linux")]
fn identity(path: &std::path::Path) -> Result<(String, String), String> {
	use std::os::unix::fs::MetadataExt;
	if !path.is_absolute() {
		return Err("Undo requires an absolute path".to_string());
	}
	let metadata = std::fs::symlink_metadata(path).map_err(|e| e.to_string())?;
	Ok((metadata.dev().to_string(), metadata.ino().to_string()))
}

#[cfg(target_os = "linux")]
fn move_no_replace(
	source: &std::path::Path,
	destination: &std::path::Path,
	expected: &(String, String),
) -> Result<(), String> {
	use std::os::unix::ffi::OsStrExt;
	if !destination.is_absolute() {
		return Err("Undo requires an absolute destination".to_string());
	}
	if &identity(source)? != expected {
		return Err("The original file has been replaced; undo was cancelled".to_string());
	}
	let source =
		std::ffi::CString::new(source.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
	let destination =
		std::ffi::CString::new(destination.as_os_str().as_bytes()).map_err(|e| e.to_string())?;
	// NOREPLACE closes the race between checking the old name and moving back.
	let result = unsafe {
		libc::renameat2(
			libc::AT_FDCWD,
			source.as_ptr(),
			libc::AT_FDCWD,
			destination.as_ptr(),
			libc::RENAME_NOREPLACE,
		)
	};
	if result == 0 {
		Ok(())
	} else {
		Err(std::io::Error::last_os_error().to_string())
	}
}

#[tauri::command]
pub async fn file_identity(path: std::path::PathBuf) -> Result<(String, String), String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || identity(&path))
			.await
			.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = path;
		Err("Safe undo is currently supported on Linux".to_string())
	}
}

#[tauri::command]
pub async fn undo_move(
	source: std::path::PathBuf,
	destination: std::path::PathBuf,
	expected: (String, String),
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || move_no_replace(&source, &destination, &expected))
			.await
			.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (source, destination, expected);
		Err("Safe undo is currently supported on Linux".to_string())
	}
}

#[tauri::command]
pub async fn undo_empty_folder(
	path: std::path::PathBuf,
	expected: (String, String),
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || {
			if identity(&path)? != expected {
				return Err("The original folder has been replaced; undo was cancelled".to_string());
			}
			std::fs::remove_dir(path).map_err(|e| e.to_string())
		})
		.await
		.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (path, expected);
		Err("Safe undo is currently supported on Linux".to_string())
	}
}

/// Removes a file or link this app just created, if it is still that entry.
///
/// A file that has gained content since is kept, because undoing "New File"
/// must never discard what the user wrote into it.
#[tauri::command]
pub async fn undo_new_file(
	path: std::path::PathBuf,
	expected: (String, String),
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || {
			if identity(&path)? != expected {
				return Err("The new file has been replaced; undo was cancelled".to_string());
			}
			let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
			if metadata.is_file() && metadata.len() > 0 {
				return Err("The new file has content now; undo was cancelled".to_string());
			}
			std::fs::remove_file(path).map_err(|e| e.to_string())
		})
		.await
		.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (path, expected);
		Err("Safe undo is currently supported on Linux".to_string())
	}
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
	use super::*;

	#[tokio::test]
	async fn undo_preserves_conflicting_files_replacements_and_nonempty_folders() {
		let root = tempfile::tempdir().unwrap();
		let source = root.path().join("new");
		let destination = root.path().join("old");
		std::fs::write(&source, "original").unwrap();
		std::fs::write(&destination, "keep").unwrap();
		let expected = identity(&source).unwrap();
		assert!(move_no_replace(&source, &destination, &expected).is_err());
		assert_eq!(std::fs::read_to_string(&destination).unwrap(), "keep");
		std::fs::remove_file(&destination).unwrap();
		move_no_replace(&source, &destination, &expected).unwrap();
		std::fs::write(&source, "replacement").unwrap();
		assert!(move_no_replace(&source, &root.path().join("elsewhere"), &expected).is_err());
		assert!(
			undo_empty_folder(root.path().to_path_buf(), identity(root.path()).unwrap())
				.await
				.is_err()
		);
		assert_eq!(std::fs::read_to_string(source).unwrap(), "replacement");
	}

	#[tokio::test]
	async fn undo_new_file_removes_empty_files_and_links_but_keeps_content() {
		let root = tempfile::tempdir().unwrap();
		let empty = root.path().join("empty");
		std::fs::write(&empty, "").unwrap();
		undo_new_file(empty.clone(), identity(&empty).unwrap())
			.await
			.unwrap();
		assert!(!empty.exists());

		let written = root.path().join("written");
		std::fs::write(&written, "").unwrap();
		let expected = identity(&written).unwrap();
		std::fs::write(&written, "notes").unwrap();
		assert!(undo_new_file(written.clone(), expected).await.is_err());
		assert!(written.exists());

		let link = root.path().join("link");
		std::os::unix::fs::symlink(root.path(), &link).unwrap();
		undo_new_file(link.clone(), identity(&link).unwrap())
			.await
			.unwrap();
		assert!(std::fs::symlink_metadata(&link).is_err());
		assert!(root.path().exists());
	}
}
