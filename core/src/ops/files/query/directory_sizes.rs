//! On-demand directory content-size calculation for Explorer.
//!
//! Directory metadata lengths are not content totals on common filesystems. This
//! query walks each direct child subtree only when Explorer explicitly requests
//! folder sizes, keeping ordinary directory navigation shallow and immediate.

use crate::{
	context::CoreContext,
	domain::addressing::WingPath,
	infra::query::{LibraryQuery, QueryError, QueryResult},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{
	collections::{HashMap, HashSet},
	io::ErrorKind,
	path::{Path, PathBuf},
	sync::{
		atomic::{AtomicU64, Ordering},
		LazyLock,
	},
};
use tokio::sync::Semaphore;

/// Input for calculating recursive content totals of a directory's child folders.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DirectorySizesInput {
	/// Physical directory whose direct child folders should be measured.
	pub path: WingPath,
}

/// Verified directory totals keyed by their physical paths.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DirectorySizesOutput {
	/// Exact recursive totals for readable direct child directories.
	pub sizes: HashMap<String, u64>,
	/// Direct child directories that could not be read completely.
	pub unavailable_paths: Vec<String>,
}

/// Limits disk walks so an optional Explorer feature cannot compete with navigation.
static DIRECTORY_SIZE_CONCURRENCY: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(1));

/// Bumped by every request; a walk stops once a newer request exists.
///
/// RPC requests are not cancelled when Explorer navigates away, so without this
/// a walk of a large tree would hold the only permit for minutes.
static DIRECTORY_SIZE_GENERATION: AtomicU64 = AtomicU64::new(0);

/// Calculates recursive content totals without following symbolic links.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DirectorySizesQuery {
	pub input: DirectorySizesInput,
}

impl LibraryQuery for DirectorySizesQuery {
	type Input = DirectorySizesInput;
	type Output = DirectorySizesOutput;

	fn from_input(input: Self::Input) -> QueryResult<Self> {
		Ok(Self { input })
	}

	async fn execute(
		self,
		_context: std::sync::Arc<CoreContext>,
		_session: crate::infra::api::SessionContext,
	) -> QueryResult<Self::Output> {
		let path = self.input.path.as_local_path().ok_or_else(|| {
			QueryError::InvalidInput("Directory sizes are only available for physical paths".into())
		})?;

		if !path.is_absolute() {
			return Err(QueryError::InvalidInput(format!(
				"Path is not absolute: {}",
				path.display()
			)));
		}

		if !tokio::fs::metadata(path)
			.await
			.map_err(|error| QueryError::io_error(path.display().to_string(), error))?
			.is_dir()
		{
			return Err(QueryError::InvalidInput(format!(
				"Path is not a directory: {}",
				path.display()
			)));
		}

		let generation = DIRECTORY_SIZE_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
		let permit = DIRECTORY_SIZE_CONCURRENCY
			.acquire()
			.await
			.map_err(|_| QueryError::Internal("Directory size worker is unavailable".into()))?;
		let path = path.to_path_buf();
		let query_path = path.clone();

		tokio::task::spawn_blocking(move || {
			// The permit lives as long as the walk, even if the request future is dropped.
			let _permit = permit;
			measure_child_directories(&query_path, &|| {
				DIRECTORY_SIZE_GENERATION.load(Ordering::SeqCst) != generation
			})
		})
		.await
		.map_err(|error| QueryError::Internal(format!("Directory size worker failed: {error}")))?
		.map_err(|error| match error.kind() {
			ErrorKind::Interrupted => QueryError::Internal(
				"Directory size request was superseded by a newer request".into(),
			),
			_ => QueryError::io_error(path.display().to_string(), error),
		})
	}
}

fn superseded() -> std::io::Error {
	std::io::Error::new(ErrorKind::Interrupted, "directory size request superseded")
}

fn measure_child_directories(
	root: &Path,
	is_superseded: &dyn Fn() -> bool,
) -> std::io::Result<DirectorySizesOutput> {
	let mut sizes = HashMap::new();
	let mut unavailable_paths = Vec::new();

	for entry in std::fs::read_dir(root)? {
		let entry = match entry {
			Ok(entry) => entry,
			Err(error) if error.kind() == ErrorKind::NotFound => continue,
			Err(error) => return Err(error),
		};
		let path = entry.path();
		match entry.file_type() {
			Ok(file_type) if file_type.is_dir() => {}
			Ok(_) => continue,
			Err(error) if error.kind() == ErrorKind::NotFound => continue,
			Err(error) => return Err(error),
		}

		match measure_directory(&path, is_superseded) {
			Ok(size) => {
				sizes.insert(path.to_string_lossy().into_owned(), size);
			}
			Err(error) if error.kind() == ErrorKind::Interrupted => return Err(error),
			// The child vanished between listing and measuring.
			Err(error) if error.kind() == ErrorKind::NotFound => {}
			Err(error) => {
				tracing::debug!(path = %path.display(), %error, "Directory size unavailable");
				unavailable_paths.push(path.to_string_lossy().into_owned());
			}
		}
	}

	Ok(DirectorySizesOutput {
		sizes,
		unavailable_paths,
	})
}

/// Sums file and symlink lengths below `root`, counting each hard-linked inode once.
///
/// Entries removed mid-walk are skipped; any other error leaves the total
/// incomplete and is returned so the caller can report the directory as unavailable.
fn measure_directory(root: &Path, is_superseded: &dyn Fn() -> bool) -> std::io::Result<u64> {
	let mut total = 0u64;
	let mut directories = vec![PathBuf::from(root)];
	let mut seen_inodes = HashSet::new();

	while let Some(directory) = directories.pop() {
		if is_superseded() {
			return Err(superseded());
		}

		let entries = match std::fs::read_dir(&directory) {
			Ok(entries) => entries,
			Err(error) if error.kind() == ErrorKind::NotFound && directory != root => continue,
			Err(error) => return Err(error),
		};

		for entry in entries {
			let entry = match entry {
				Ok(entry) => entry,
				Err(error) if error.kind() == ErrorKind::NotFound => continue,
				Err(error) => return Err(error),
			};
			let path = entry.path();
			let metadata = match std::fs::symlink_metadata(&path) {
				Ok(metadata) => metadata,
				Err(error) if error.kind() == ErrorKind::NotFound => continue,
				Err(error) => return Err(error),
			};

			if metadata.is_dir() {
				directories.push(path);
			} else if is_first_link(&metadata, &mut seen_inodes) {
				total = total
					.checked_add(metadata.len())
					.ok_or_else(|| std::io::Error::other("Directory content size exceeds u64"))?;
			}
		}
	}

	Ok(total)
}

#[cfg(unix)]
fn is_first_link(metadata: &std::fs::Metadata, seen_inodes: &mut HashSet<(u64, u64)>) -> bool {
	use std::os::unix::fs::MetadataExt;

	metadata.nlink() <= 1 || seen_inodes.insert((metadata.dev(), metadata.ino()))
}

#[cfg(not(unix))]
fn is_first_link(_metadata: &std::fs::Metadata, _seen_inodes: &mut HashSet<(u64, u64)>) -> bool {
	true
}

crate::register_library_query!(DirectorySizesQuery, "files.directory_sizes");

#[cfg(test)]
mod tests {
	use super::*;

	fn never() -> bool {
		false
	}

	#[test]
	fn measures_nested_files_without_counting_directory_metadata() {
		let temporary = tempfile::tempdir().unwrap();
		let child = temporary.path().join("child");
		std::fs::create_dir(&child).unwrap();
		std::fs::write(child.join("top.txt"), b"abc").unwrap();
		std::fs::create_dir(child.join("nested")).unwrap();
		std::fs::write(child.join("nested").join("bottom.txt"), b"12345").unwrap();

		let output = measure_child_directories(temporary.path(), &never).unwrap();

		assert_eq!(output.sizes.get(child.to_string_lossy().as_ref()), Some(&8));
		assert!(output.unavailable_paths.is_empty());
	}

	#[cfg(unix)]
	#[test]
	fn counts_hard_linked_content_once() {
		let temporary = tempfile::tempdir().unwrap();
		let child = temporary.path().join("child");
		std::fs::create_dir(&child).unwrap();
		std::fs::write(child.join("original.txt"), b"1234").unwrap();
		std::fs::hard_link(child.join("original.txt"), child.join("link.txt")).unwrap();

		let output = measure_child_directories(temporary.path(), &never).unwrap();

		assert_eq!(output.sizes.get(child.to_string_lossy().as_ref()), Some(&4));
	}

	#[cfg(unix)]
	#[test]
	fn reports_unreadable_child_without_failing_siblings() {
		use std::os::unix::fs::PermissionsExt;

		let temporary = tempfile::tempdir().unwrap();
		let readable = temporary.path().join("readable");
		let locked = temporary.path().join("locked");
		std::fs::create_dir(&readable).unwrap();
		std::fs::write(readable.join("file.txt"), b"12").unwrap();
		std::fs::create_dir_all(locked.join("inner")).unwrap();
		std::fs::set_permissions(locked.join("inner"), std::fs::Permissions::from_mode(0o000))
			.unwrap();

		let output = measure_child_directories(temporary.path(), &never);
		std::fs::set_permissions(locked.join("inner"), std::fs::Permissions::from_mode(0o755))
			.unwrap();
		let output = output.unwrap();

		assert_eq!(
			output.sizes.get(readable.to_string_lossy().as_ref()),
			Some(&2)
		);
		// Root bypasses permission bits, so only assert the locked case for normal users.
		if !output.sizes.contains_key(locked.to_string_lossy().as_ref()) {
			assert_eq!(
				output.unavailable_paths,
				vec![locked.to_string_lossy().into_owned()]
			);
		}
	}

	#[test]
	fn stops_when_superseded() {
		let temporary = tempfile::tempdir().unwrap();
		std::fs::create_dir(temporary.path().join("child")).unwrap();

		let error = measure_child_directories(temporary.path(), &|| true).unwrap_err();

		assert_eq!(error.kind(), ErrorKind::Interrupted);
	}
}
