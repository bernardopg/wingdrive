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

#[cfg(target_os = "linux")]
fn content_hash(path: &std::path::Path) -> Result<String, String> {
	if !path.is_absolute() {
		return Err("Undo requires an absolute path".to_string());
	}
	let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
	let mut hasher = blake3::Hasher::new();
	hasher.update_reader(&mut file).map_err(|e| e.to_string())?;
	Ok(hasher.finalize().to_hex().to_string())
}

#[tauri::command]
pub async fn file_content_hash(path: std::path::PathBuf) -> Result<String, String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || content_hash(&path))
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
/// Empty files must remain empty, and template copies must retain their
/// original content so undo cannot discard subsequent edits.
#[tauri::command]
pub async fn undo_new_file(
	path: std::path::PathBuf,
	expected: (String, String),
	expected_hash: Option<String>,
) -> Result<(), String> {
	#[cfg(target_os = "linux")]
	{
		tokio::task::spawn_blocking(move || {
			if identity(&path)? != expected {
				return Err("The new file has been replaced; undo was cancelled".to_string());
			}
			let metadata = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
			if let Some(hash) = expected_hash {
				if !metadata.is_file()
					|| content_hash(&path)? != hash
					|| identity(&path)? != expected
				{
					return Err("The new file has changed; undo was cancelled".to_string());
				}
			} else if metadata.is_file() && metadata.len() > 0 {
				return Err("The new file has content now; undo was cancelled".to_string());
			}
			std::fs::remove_file(path).map_err(|e| e.to_string())
		})
		.await
		.map_err(|e| e.to_string())?
	}
	#[cfg(not(target_os = "linux"))]
	{
		let _ = (path, expected, expected_hash);
		Err("Safe undo is currently supported on Linux".to_string())
	}
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
	use super::*;

	#[tokio::test]
	async fn move_keeps_existing_directories_and_links_to_directories() {
		let root = tempfile::tempdir().unwrap();
		let source = root.path().join("source");
		let directory = root.path().join("directory");
		let link = root.path().join("link");
		std::fs::write(&source, b"original").unwrap();
		std::fs::create_dir(&directory).unwrap();
		std::fs::write(directory.join("new-data"), b"keep").unwrap();
		std::os::unix::fs::symlink(&directory, &link).unwrap();
		let expected = file_identity(source.clone()).await.unwrap();
		for destination in [directory.clone(), link.clone()] {
			assert!(undo_move(source.clone(), destination, expected.clone())
				.await
				.is_err());
			assert_eq!(std::fs::read(&source).unwrap(), b"original");
			assert_eq!(std::fs::read(directory.join("new-data")).unwrap(), b"keep");
		}
		assert_eq!(std::fs::read_link(link).unwrap(), directory);
	}

	#[tokio::test]
	async fn move_keeps_an_existing_dangling_link_and_its_source() {
		let root = tempfile::tempdir().unwrap();
		let source = root.path().join("source");
		let destination = root.path().join("destination");
		let missing = root.path().join("missing");
		std::fs::write(&source, b"original").unwrap();
		std::os::unix::fs::symlink(&missing, &destination).unwrap();
		let expected = file_identity(source.clone()).await.unwrap();

		assert!(undo_move(source.clone(), destination.clone(), expected)
			.await
			.is_err());
		assert_eq!(std::fs::read(&source).unwrap(), b"original");
		assert_eq!(std::fs::read_link(&destination).unwrap(), missing);
	}

	#[tokio::test]
	async fn move_restores_the_link_without_moving_its_target() {
		let root = tempfile::tempdir().unwrap();
		let target = root.path().join("target");
		let source = root.path().join("link");
		let destination = root.path().join("restored");
		std::fs::write(&target, b"keep target").unwrap();
		std::os::unix::fs::symlink(&target, &source).unwrap();
		let expected = file_identity(source.clone()).await.unwrap();
		assert_ne!(expected, file_identity(target.clone()).await.unwrap());

		undo_move(source.clone(), destination.clone(), expected)
			.await
			.unwrap();
		assert!(std::fs::symlink_metadata(&source).is_err());
		assert_eq!(std::fs::read_link(destination).unwrap(), target);
		assert_eq!(std::fs::read(target).unwrap(), b"keep target");
	}

	#[tokio::test]
	async fn replacing_a_new_file_with_a_link_cancels_both_undo_operations() {
		let root = tempfile::tempdir().unwrap();
		let path = root.path().join("created");
		let original = root.path().join("original");
		let destination = root.path().join("restored");
		std::fs::write(&path, []).unwrap();
		let expected = file_identity(path.clone()).await.unwrap();
		std::fs::rename(&path, &original).unwrap();
		std::os::unix::fs::symlink(&original, &path).unwrap();

		assert!(undo_new_file(path.clone(), expected.clone(), None)
			.await
			.is_err());
		assert!(undo_move(path.clone(), destination.clone(), expected)
			.await
			.is_err());
		assert_eq!(std::fs::read_link(path).unwrap(), original);
		assert_eq!(std::fs::read(original).unwrap(), b"");
		assert!(!destination.exists());
	}

	#[tokio::test]
	async fn folder_undo_keeps_a_replacement_even_when_it_is_empty() {
		let root = tempfile::tempdir().unwrap();
		let path = root.path().join("folder");
		let original = root.path().join("original");
		std::fs::create_dir(&path).unwrap();
		let expected = file_identity(path.clone()).await.unwrap();
		std::fs::rename(&path, &original).unwrap();
		std::fs::create_dir(&path).unwrap();

		assert!(undo_empty_folder(path.clone(), expected).await.is_err());
		assert!(path.is_dir());
		assert!(original.is_dir());
	}

	#[tokio::test]
	async fn link_undo_removes_a_dangling_link_but_a_template_hash_rejects_it() {
		let root = tempfile::tempdir().unwrap();
		let path = root.path().join("link");
		let missing = root.path().join("missing");
		std::os::unix::fs::symlink(&missing, &path).unwrap();
		let expected = file_identity(path.clone()).await.unwrap();

		assert!(
			undo_new_file(path.clone(), expected.clone(), Some("hash".into()))
				.await
				.is_err()
		);
		assert_eq!(std::fs::read_link(&path).unwrap(), missing);
		undo_new_file(path.clone(), expected, None).await.unwrap();
		assert!(std::fs::symlink_metadata(path).is_err());
		assert!(!missing.exists());
	}

	#[tokio::test]
	async fn failed_move_to_a_missing_parent_or_nul_path_keeps_the_source() {
		let root = tempfile::tempdir().unwrap();
		let source = root.path().join("source");
		std::fs::write(&source, b"keep").unwrap();
		let expected = file_identity(source.clone()).await.unwrap();
		for destination in [
			root.path().join("missing/file"),
			root.path().join("nul\0file"),
		] {
			assert!(undo_move(source.clone(), destination, expected.clone())
				.await
				.is_err());
			assert_eq!(std::fs::read(&source).unwrap(), b"keep");
		}
	}

	#[tokio::test]
	async fn template_hash_detects_same_size_edits_and_hashes_empty_files() {
		let root = tempfile::tempdir().unwrap();
		let path = root.path().join("template");
		std::fs::write(&path, []).unwrap();
		assert_eq!(
			file_content_hash(path.clone()).await.unwrap(),
			blake3::hash(b"").to_hex().to_string()
		);
		std::fs::write(&path, b"original").unwrap();
		let expected = file_identity(path.clone()).await.unwrap();
		let hash = file_content_hash(path.clone()).await.unwrap();
		std::fs::write(&path, b"modified").unwrap();

		assert!(undo_new_file(path.clone(), expected, Some(hash))
			.await
			.is_err());
		assert_eq!(std::fs::read(path).unwrap(), b"modified");
		assert!(file_content_hash(root.path().join("missing"))
			.await
			.is_err());
	}

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
		undo_new_file(empty.clone(), identity(&empty).unwrap(), None)
			.await
			.unwrap();
		assert!(!empty.exists());

		let written = root.path().join("written");
		std::fs::write(&written, "").unwrap();
		let expected = identity(&written).unwrap();
		std::fs::write(&written, "notes").unwrap();
		assert!(undo_new_file(written.clone(), expected, None)
			.await
			.is_err());
		assert!(written.exists());

		let link = root.path().join("link");
		std::os::unix::fs::symlink(root.path(), &link).unwrap();
		undo_new_file(link.clone(), identity(&link).unwrap(), None)
			.await
			.unwrap();
		assert!(std::fs::symlink_metadata(&link).is_err());
		assert!(root.path().exists());
	}

	#[tokio::test]
	async fn undo_template_keeps_changed_or_replaced_files() {
		let root = tempfile::tempdir().unwrap();
		let path = root.path().join("template-copy");
		std::fs::write(&path, b"original").unwrap();
		let expected = identity(&path).unwrap();
		let hash = file_content_hash(path.clone()).await.unwrap();
		std::fs::write(&path, b"modified").unwrap();
		assert!(
			undo_new_file(path.clone(), expected.clone(), Some(hash.clone()))
				.await
				.is_err()
		);
		assert_eq!(std::fs::read(&path).unwrap(), b"modified");
		std::fs::rename(&path, root.path().join("original-entry")).unwrap();
		std::fs::write(&path, b"original").unwrap();
		assert!(undo_new_file(path.clone(), expected, Some(hash.clone()))
			.await
			.is_err());
		let expected = identity(&path).unwrap();
		undo_new_file(path.clone(), expected, Some(hash))
			.await
			.unwrap();
		assert!(!path.exists());
	}

	#[tokio::test]
	async fn undo_rejects_relative_paths_and_missing_or_directory_entries() {
		let root = tempfile::tempdir().unwrap();
		let relative = std::path::PathBuf::from("relative");
		assert!(file_identity(relative.clone()).await.is_err());
		assert!(file_content_hash(relative.clone()).await.is_err());
		let file = root.path().join("file");
		std::fs::write(&file, "data").unwrap();
		let expected = identity(&file).unwrap();
		assert!(move_no_replace(&file, &relative, &expected).is_err());
		assert!(undo_move(relative, root.path().join("x"), expected.clone())
			.await
			.is_err());
		assert!(file.exists());

		let missing = root.path().join("missing");
		assert!(file_identity(missing.clone()).await.is_err());
		assert!(undo_new_file(missing.clone(), expected.clone(), None)
			.await
			.is_err());
		assert!(undo_empty_folder(missing, expected).await.is_err());

		// A moved file goes back when the original name is still free.
		let moved = root.path().join("moved");
		std::fs::rename(&file, &moved).unwrap();
		let expected = file_identity(moved.clone()).await.unwrap();
		undo_move(moved.clone(), file.clone(), expected)
			.await
			.unwrap();
		assert_eq!(std::fs::read_to_string(&file).unwrap(), "data");
		assert!(!moved.exists());

		// A directory is never removed as a new file, with or without a hash.
		let folder = root.path().join("folder");
		std::fs::create_dir(&folder).unwrap();
		let expected = identity(&folder).unwrap();
		assert!(undo_new_file(folder.clone(), expected.clone(), None)
			.await
			.is_err());
		assert!(
			undo_new_file(folder.clone(), expected.clone(), Some("hash".into()))
				.await
				.is_err()
		);
		assert!(file_content_hash(folder.clone()).await.is_err());
		undo_empty_folder(folder.clone(), expected).await.unwrap();
		assert!(!folder.exists());
	}
}
