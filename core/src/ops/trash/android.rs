//! Recoverable trash on Android storage volumes accessible to WingDrive.

use super::TrashEntry;
use std::{
	ffi::CString,
	fs, io,
	os::unix::ffi::OsStrExt,
	path::{Path, PathBuf},
	sync::Mutex,
};
use uuid::Uuid;

const FOLDER: &str = ".wingdrive-trash";
static OPERATIONS: Mutex<()> = Mutex::new(());

fn rename_exclusive(source: &Path, destination: &Path) -> io::Result<()> {
	let from = CString::new(source.as_os_str().as_bytes())?;
	let to = CString::new(destination.as_os_str().as_bytes())?;
	// SAFETY: both path pointers refer to NUL-terminated strings valid for this call.
	let result = unsafe {
		libc::syscall(
			libc::SYS_renameat2,
			libc::AT_FDCWD,
			from.as_ptr(),
			libc::AT_FDCWD,
			to.as_ptr(),
			libc::RENAME_NOREPLACE,
		)
	};
	if result == 0 {
		Ok(())
	} else {
		Err(io::Error::last_os_error())
	}
}

#[cfg(target_os = "android")]
fn storage_root(path: &Path) -> Option<PathBuf> {
	let relative = path.strip_prefix("/storage").ok()?;
	let mut parts = relative.iter();
	let volume = parts.next()?;
	let mut root = Path::new("/storage").join(volume);
	if volume == "emulated" {
		root.push(parts.next()?);
	}
	Some(root)
}

#[cfg(target_os = "android")]
pub(super) fn put(data_dir: &Path, path: &Path) -> io::Result<()> {
	let name = path.file_name().ok_or(io::ErrorKind::InvalidInput)?;
	let parent = path
		.parent()
		.ok_or(io::ErrorKind::InvalidInput)?
		.canonicalize()?;
	let root = storage_root(&parent).ok_or(io::ErrorKind::Unsupported)?;
	put_at(data_dir, &root, &parent.join(name))
}

fn put_at(data_dir: &Path, root: &Path, path: &Path) -> io::Result<()> {
	let _guard = OPERATIONS.lock().unwrap_or_else(|error| error.into_inner());
	let folder = root.join(FOLDER);
	if path.starts_with(&folder) || folder.starts_with(path) {
		return Err(io::ErrorKind::InvalidInput.into());
	}
	fs::create_dir_all(&folder)?;
	if fs::symlink_metadata(&folder)?.file_type().is_symlink() {
		return Err(io::ErrorKind::InvalidInput.into());
	}
	let metadata = fs::symlink_metadata(path)?;
	let id = Uuid::new_v4().to_string();
	let entry = TrashEntry {
		id: id.clone(),
		name: path
			.file_name()
			.ok_or(io::ErrorKind::InvalidInput)?
			.to_string_lossy()
			.into_owned(),
		original_path: path.to_str().ok_or(io::ErrorKind::InvalidInput)?.to_owned(),
		deleted_at: chrono::Utc::now().timestamp(),
		is_dir: metadata.is_dir(),
		size: metadata.is_file().then(|| metadata.len()),
	};
	let registry = data_dir.join(FOLDER);
	fs::create_dir_all(&registry)?;
	let record = registry.join(&id);
	let payload = folder.join(&id);

	// Publish the record before moving the file so a process restart can recover it.
	let mut file = fs::OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(&record)?;
	let result = (|| {
		serde_json::to_writer(&mut file, &entry)?;
		file.sync_all()?;
		rename_exclusive(path, &payload)
	})();
	if result.is_err() {
		let _ = fs::remove_file(record);
	}
	result
}

fn records(
	data_dir: &Path,
	root_for: impl Fn(&Path) -> Option<PathBuf>,
) -> io::Result<Vec<(TrashEntry, PathBuf)>> {
	let dir = match fs::read_dir(data_dir.join(FOLDER)) {
		Ok(dir) => dir,
		Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
		Err(error) => return Err(error),
	};
	let mut items = Vec::new();
	for record in dir {
		let record = record?;
		let id = record.file_name();
		let Some(id) = id.to_str().filter(|id| Uuid::parse_str(id).is_ok()) else {
			continue;
		};
		let entry: TrashEntry = match serde_json::from_reader(fs::File::open(record.path())?) {
			Ok(entry) => entry,
			Err(error) => {
				tracing::warn!(%error, "Skipping an unreadable Android trash record");
				continue;
			}
		};
		if entry.id != id {
			continue;
		}
		let Some(root) = root_for(Path::new(&entry.original_path)) else {
			continue;
		};
		let payload = root.join(FOLDER).join(id);
		if fs::symlink_metadata(&payload).is_ok() {
			items.push((entry, payload));
		}
	}
	items.sort_by_key(|(entry, _)| std::cmp::Reverse(entry.deleted_at));
	Ok(items)
}

#[cfg(target_os = "android")]
pub(super) fn list(data_dir: &Path) -> Result<Vec<TrashEntry>, String> {
	let _guard = OPERATIONS.lock().unwrap_or_else(|error| error.into_inner());
	records(data_dir, storage_root)
		.map(|items| items.into_iter().map(|(entry, _)| entry).collect())
		.map_err(|error| error.to_string())
}

#[cfg(target_os = "android")]
pub(super) fn update(
	data_dir: &Path,
	ids: &[String],
	paths: &[String],
	restore: bool,
) -> Result<u32, String> {
	let _guard = OPERATIONS.lock().unwrap_or_else(|error| error.into_inner());
	update_at(data_dir, ids, paths, restore, storage_root).map_err(|error| error.to_string())
}

fn update_at(
	data_dir: &Path,
	ids: &[String],
	paths: &[String],
	restore: bool,
	root_for: impl Fn(&Path) -> Option<PathBuf>,
) -> io::Result<u32> {
	let mut count = 0;
	let mut restored_paths = std::collections::HashSet::new();
	for (entry, payload) in records(data_dir, root_for)? {
		if !ids.contains(&entry.id)
			&& !(restore
				&& paths.contains(&entry.original_path)
				&& restored_paths.insert(entry.original_path.clone()))
		{
			continue;
		}
		if restore {
			let destination = Path::new(&entry.original_path);
			let parent = destination.parent().ok_or(io::ErrorKind::InvalidInput)?;
			if parent.canonicalize()? != parent {
				return Err(io::Error::new(
					io::ErrorKind::InvalidInput,
					"the original folder has moved",
				));
			}
			rename_exclusive(&payload, destination)?;
		} else if fs::symlink_metadata(&payload)?.is_dir() {
			fs::remove_dir_all(&payload)?;
		} else {
			fs::remove_file(&payload)?;
		}
		let _ = fs::remove_file(data_dir.join(FOLDER).join(entry.id));
		count += 1;
	}
	Ok(count)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn trash_restore_and_purge_preserve_an_existing_destination() {
		let volume = tempfile::tempdir().unwrap();
		let data = tempfile::tempdir().unwrap();
		let root = volume.path();
		let original = root.join("file.txt");
		let root_for = |_: &Path| Some(root.to_path_buf());
		fs::write(&original, "original").unwrap();
		put_at(data.path(), root, &original).unwrap();
		assert!(!original.exists());
		let id = records(data.path(), root_for).unwrap().remove(0).0.id;
		fs::write(&original, "replacement").unwrap();
		assert_eq!(
			update_at(data.path(), &[id.clone()], &[], true, root_for)
				.unwrap_err()
				.kind(),
			io::ErrorKind::AlreadyExists
		);
		assert_eq!(fs::read_to_string(&original).unwrap(), "replacement");
		fs::remove_file(&original).unwrap();
		assert_eq!(
			update_at(data.path(), &[id], &[], true, root_for).unwrap(),
			1
		);
		assert_eq!(fs::read_to_string(&original).unwrap(), "original");
		put_at(data.path(), root, &original).unwrap();
		let id = records(data.path(), root_for).unwrap().remove(0).0.id;
		assert_eq!(
			update_at(data.path(), &[id], &[], false, root_for).unwrap(),
			1
		);
		assert!(records(data.path(), root_for).unwrap().is_empty());
	}
}
