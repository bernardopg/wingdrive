//! Browse, restore and empty the system trash.
//!
//! Deleting without `permanent` moves files to the platform trash, so these
//! operations give the user a way back: list what is there, put items back
//! where they came from, or remove them for good. Listing and restoring are
//! only exposed by the platform on Linux (freedesktop trash) and Windows.

use crate::{
	context::CoreContext,
	infra::{
		action::{error::ActionError, CoreAction},
		api::SessionContext,
		query::{CoreQuery, QueryError, QueryResult},
	},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;

/// One item in the system trash.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashEntry {
	/// Platform identifier; pass it back to restore or purge the item
	pub id: String,
	pub name: String,
	/// Where the item lived before it was trashed
	pub original_path: String,
	/// Seconds since the Unix epoch
	pub deleted_at: i64,
	pub is_dir: bool,
	/// Size in bytes for files, `None` for folders
	pub size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashListInput {}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashListOutput {
	/// Newest first
	pub items: Vec<TrashEntry>,
	/// False where the platform offers no way to read its trash
	pub supported: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashRestoreInput {
	/// Items to restore, by `TrashEntry::id`
	pub ids: Vec<String>,
	/// Items to restore by their original path; the newest copy of each wins.
	/// Undo uses this, since a delete does not report trash identifiers.
	#[serde(default)]
	#[specta(optional)]
	pub original_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashRestoreOutput {
	pub restored: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashPurgeInput {
	/// Items to delete for good, by `TrashEntry::id`
	pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TrashPurgeOutput {
	pub purged: u32,
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
mod platform {
	use super::TrashEntry;
	use std::path::Path;
	use trash::{os_limited, TrashItem, TrashItemSize};

	pub const SUPPORTED: bool = true;

	pub fn list() -> Result<Vec<TrashEntry>, String> {
		let mut items = os_limited::list().map_err(|e| e.to_string())?;
		items.sort_by_key(|item| std::cmp::Reverse(item.time_deleted));
		Ok(items
			.iter()
			.map(|item| {
				let size = os_limited::metadata(item).ok().map(|m| m.size);
				TrashEntry {
					id: item.id.to_string_lossy().into_owned(),
					name: item.name.to_string_lossy().into_owned(),
					original_path: item.original_path().to_string_lossy().into_owned(),
					deleted_at: item.time_deleted,
					is_dir: matches!(size, Some(TrashItemSize::Entries(_))),
					size: match size {
						Some(TrashItemSize::Bytes(bytes)) => Some(bytes),
						_ => None,
					},
				}
			})
			.collect())
	}

	/// Items named by id, plus the newest item for each original path.
	fn select(ids: &[String], paths: &[String]) -> Result<Vec<TrashItem>, String> {
		let mut items = os_limited::list().map_err(|e| e.to_string())?;
		items.sort_by_key(|item| std::cmp::Reverse(item.time_deleted));
		let wanted: Vec<_> = paths
			.iter()
			.map(|p| super::trash_location(Path::new(p)))
			.collect();
		let mut chosen: Vec<TrashItem> = Vec::new();
		for item in items {
			let original = item.original_path();
			let by_id = ids
				.iter()
				.any(|id| item.id.to_string_lossy() == id.as_str());
			let by_path = wanted.iter().any(|p| *p == original)
				&& !chosen.iter().any(|c| c.original_path() == original);
			if by_id || by_path {
				chosen.push(item);
			}
		}
		Ok(chosen)
	}

	pub fn restore(ids: &[String], paths: &[String]) -> Result<u32, String> {
		let items = select(ids, paths)?;
		let count = items.len() as u32;
		os_limited::restore_all(items).map_err(|e| e.to_string())?;
		Ok(count)
	}

	/// The newest trash item of each original path.
	pub fn newest_ids(
		paths: &[std::path::PathBuf],
	) -> Result<std::collections::HashMap<std::path::PathBuf, String>, String> {
		let mut items = os_limited::list().map_err(|e| e.to_string())?;
		items.sort_by_key(|item| std::cmp::Reverse(item.time_deleted));
		let wanted: Vec<_> = paths
			.iter()
			.map(|p| (super::trash_location(p), p))
			.collect();
		let mut ids = std::collections::HashMap::new();
		for item in items {
			let original = item.original_path();
			for (location, requested) in &wanted {
				if *location == original {
					ids.entry((*requested).clone())
						.or_insert_with(|| item.id.to_string_lossy().into_owned());
				}
			}
		}
		Ok(ids)
	}

	pub fn purge(ids: &[String]) -> Result<u32, String> {
		let items = select(ids, &[])?;
		let count = items.len() as u32;
		os_limited::purge_all(items).map_err(|e| e.to_string())?;
		Ok(count)
	}
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
mod platform {
	use super::TrashEntry;

	pub const SUPPORTED: bool = false;
	const UNSUPPORTED: &str = "the system trash cannot be read on this platform";

	pub fn list() -> Result<Vec<TrashEntry>, String> {
		Ok(Vec::new())
	}

	pub fn restore(_ids: &[String], _paths: &[String]) -> Result<u32, String> {
		Err(UNSUPPORTED.to_string())
	}

	pub fn purge(_ids: &[String]) -> Result<u32, String> {
		Err(UNSUPPORTED.to_string())
	}

	pub fn newest_ids(
		_paths: &[std::path::PathBuf],
	) -> Result<std::collections::HashMap<std::path::PathBuf, String>, String> {
		Ok(std::collections::HashMap::new())
	}
}

/// The path the system trash records for an item trashed from `path`.
///
/// The `trash` crate canonicalizes the parent folder before it moves an item,
/// so a path whose parent crosses a symbolic link is recorded under the link's
/// target. Comparing journal paths with this form finds those items again; the
/// item itself is not resolved, since it is no longer there after trashing.
pub(crate) fn trash_location(path: &std::path::Path) -> std::path::PathBuf {
	match (path.parent(), path.file_name()) {
		(Some(parent), Some(name)) => match std::fs::canonicalize(parent) {
			Ok(parent) => parent.join(name),
			Err(_) => path.to_path_buf(),
		},
		_ => path.to_path_buf(),
	}
}

async fn blocking<T: Send + 'static>(
	work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
	tokio::task::spawn_blocking(work)
		.await
		.map_err(|e| e.to_string())?
}

/// Trash identifiers of items a job just trashed, by original path.
///
/// Empty where the platform cannot list its trash.
pub(crate) async fn trashed_ids(
	paths: Vec<std::path::PathBuf>,
) -> std::collections::HashMap<std::path::PathBuf, String> {
	blocking(move || platform::newest_ids(&paths))
		.await
		.unwrap_or_default()
}

/// Puts one trashed item back where it came from.
///
/// With no identifier, the newest item trashed from `original` is restored.
pub(crate) async fn restore_one(
	trash_id: Option<String>,
	original: std::path::PathBuf,
) -> Result<(), String> {
	let restored = blocking(move || match trash_id {
		Some(id) => platform::restore(&[id], &[]),
		None => platform::restore(&[], &[original.to_string_lossy().into_owned()]),
	})
	.await?;
	if restored == 0 {
		return Err("it is no longer in the trash".to_string());
	}
	Ok(())
}

pub struct TrashListQuery;

impl CoreQuery for TrashListQuery {
	type Input = TrashListInput;
	type Output = TrashListOutput;

	fn from_input(_input: Self::Input) -> QueryResult<Self> {
		Ok(Self)
	}

	async fn execute(
		self,
		_context: Arc<CoreContext>,
		_session: SessionContext,
	) -> QueryResult<Self::Output> {
		let items = blocking(platform::list)
			.await
			.map_err(QueryError::Internal)?;
		Ok(TrashListOutput {
			items,
			supported: platform::SUPPORTED,
		})
	}
}

crate::register_core_query!(TrashListQuery, "trash.list");

pub struct TrashRestoreAction {
	input: TrashRestoreInput,
}

impl CoreAction for TrashRestoreAction {
	type Input = TrashRestoreInput;
	type Output = TrashRestoreOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.ids.is_empty() && input.original_paths.is_empty() {
			return Err("nothing to restore".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(self, _context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let TrashRestoreInput {
			ids,
			original_paths,
		} = self.input;
		let restored = blocking(move || platform::restore(&ids, &original_paths))
			.await
			.map_err(ActionError::Internal)?;
		Ok(TrashRestoreOutput { restored })
	}

	fn action_kind(&self) -> &'static str {
		"trash.restore"
	}
}

crate::register_core_action!(TrashRestoreAction, "trash.restore");

pub struct TrashPurgeAction {
	input: TrashPurgeInput,
}

impl CoreAction for TrashPurgeAction {
	type Input = TrashPurgeInput;
	type Output = TrashPurgeOutput;

	fn from_input(input: Self::Input) -> Result<Self, String> {
		if input.ids.is_empty() {
			return Err("nothing to purge".to_string());
		}
		Ok(Self { input })
	}

	async fn execute(self, _context: Arc<CoreContext>) -> Result<Self::Output, ActionError> {
		let ids = self.input.ids;
		let purged = blocking(move || platform::purge(&ids))
			.await
			.map_err(ActionError::Internal)?;
		Ok(TrashPurgeOutput { purged })
	}

	fn action_kind(&self) -> &'static str {
		"trash.purge"
	}
}

crate::register_core_action!(TrashPurgeAction, "trash.purge");

#[cfg(all(test, target_os = "linux"))]
mod tests {
	use super::platform;

	/// The tests point XDG_DATA_HOME, which is process-wide, at their own trash.
	static TRASH_ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

	#[test]
	fn trash_location_resolves_a_symlinked_parent_only() {
		let root = tempfile::tempdir().unwrap();
		let real = root.path().join("real");
		std::fs::create_dir(&real).unwrap();
		let link = root.path().join("link");
		std::os::unix::fs::symlink(&real, &link).unwrap();
		let real = std::fs::canonicalize(&real).unwrap();
		assert_eq!(
			super::trash_location(&link.join("gone.txt")),
			real.join("gone.txt")
		);
		// The item may itself be a link; only its parent is resolved.
		std::os::unix::fs::symlink(&real, real.join("self")).unwrap();
		assert_eq!(super::trash_location(&real.join("self")), real.join("self"));
		let missing = root.path().join("missing/child");
		assert_eq!(super::trash_location(&missing), missing);
	}

	#[tokio::test]
	async fn items_trashed_through_a_symlinked_folder_are_found_and_restored() {
		let _env = TRASH_ENV.lock().unwrap_or_else(|e| e.into_inner());
		let root = tempfile::tempdir().unwrap();
		std::env::set_var("XDG_DATA_HOME", root.path());
		let real = root.path().join("real");
		std::fs::create_dir(&real).unwrap();
		let link = root.path().join("link");
		std::os::unix::fs::symlink(&real, &link).unwrap();
		let file = link.join("notes.txt");
		std::fs::write(&file, b"hello").unwrap();

		trash::delete(&file).unwrap();
		let ids = super::trashed_ids(vec![file.clone()]).await;
		assert!(
			ids.contains_key(&file),
			"the journal path finds its trash item"
		);

		// Without an id, undo falls back to the newest item of the path.
		super::restore_one(None, file.clone()).await.unwrap();
		assert_eq!(std::fs::read(&file).unwrap(), b"hello");

		trash::delete(&file).unwrap();
		let ids = super::trashed_ids(vec![file.clone()]).await;
		super::restore_one(ids.get(&file).cloned(), file.clone())
			.await
			.unwrap();
		assert_eq!(std::fs::read(real.join("notes.txt")).unwrap(), b"hello");
	}

	#[test]
	fn restores_by_original_path_and_purges_by_id() {
		let _env = TRASH_ENV.lock().unwrap_or_else(|e| e.into_inner());
		// The home trash follows XDG_DATA_HOME; keep the file on the same
		// mount so the test never touches the user's real trash.
		let root = tempfile::tempdir().unwrap();
		std::env::set_var("XDG_DATA_HOME", root.path());
		let file = root.path().join("notes.txt");
		std::fs::write(&file, b"hello").unwrap();
		let original = file.to_string_lossy().into_owned();

		trash::delete(&file).unwrap();
		assert!(!file.exists());
		let listed = platform::list().unwrap();
		let entry = listed.iter().find(|e| e.original_path == original).unwrap();
		assert_eq!(entry.size, Some(5));

		assert_eq!(platform::restore(&[], &[original.clone()]).unwrap(), 1);
		assert_eq!(std::fs::read(&file).unwrap(), b"hello");

		trash::delete(&file).unwrap();
		let id = platform::list()
			.unwrap()
			.into_iter()
			.find(|e| e.original_path == original)
			.unwrap()
			.id;
		assert_eq!(platform::purge(&[id]).unwrap(), 1);
		assert!(platform::list()
			.unwrap()
			.iter()
			.all(|e| e.original_path != original));
	}
}
