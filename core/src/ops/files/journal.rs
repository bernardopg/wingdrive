//! # Journal of what file jobs changed
//!
//! Copy, move and trash jobs write one [`Effect`] per item they changed, so an
//! undo job can reverse exactly that and nothing else. The journal lives in the
//! library's job database: it describes this device's disk and is never synced.
//!
//! Each effect that left something behind carries a [`Seal`], a fingerprint of
//! the item right after the job finished with it. Undo compares the seal with
//! the disk first and leaves alone anything that changed in between.
//!
//! A file that a job replaces is not deleted. It moves to a stash folder next
//! to it (`.wing-undo-<job>/`), where undo finds it. Stashes and journal rows
//! older than [`RETENTION_DAYS`] are removed when the next file job starts.

use std::{
	path::{Path, PathBuf},
	sync::atomic::{AtomicI32, Ordering},
};

use chrono::{DateTime, Duration, Utc};
use sea_orm::{
	ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
	QueryOrder, QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infra::job::{context::JobContext, database::journal};

/// How long a replaced file and its journal rows are kept.
pub const RETENTION_DAYS: i64 = 7;

/// Rows read to build the list of recent operations.
const HISTORY_SCAN_ROWS: u64 = 2000;

/// One change a file job made to the disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Effect {
	/// A copy created `path`.
	Created { path: PathBuf, seal: Seal },
	/// `from` was moved or renamed to `to`.
	Moved {
		from: PathBuf,
		to: PathBuf,
		seal: Seal,
	},
	/// The item that was at `path` is kept at `stash`.
	Replaced { path: PathBuf, stash: PathBuf },
	/// `original` went to the system trash.
	Trashed {
		original: PathBuf,
		trash_id: Option<String>,
	},
	/// The job wrote into `path`, which already existed. Nothing records what
	/// was there, so this cannot be undone.
	Merged { path: PathBuf },
}

impl Effect {
	/// The path a person would recognise this effect by.
	pub fn subject(&self) -> &Path {
		match self {
			Self::Created { path, .. } | Self::Replaced { path, .. } | Self::Merged { path } => {
				path
			}
			Self::Moved { to, .. } => to,
			Self::Trashed { original, .. } => original,
		}
	}
}

/// Fingerprint of a file or folder right after a job finished with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Seal {
	is_dir: bool,
	/// Device and inode on Unix, zero elsewhere.
	device: u64,
	inode: u64,
	/// Files and links, folders included.
	entries: u64,
	/// Bytes of every file and link.
	size: u64,
	/// Latest change time of anything in the tree, in nanoseconds since the
	/// Unix epoch. Inode change time on Unix, modification time elsewhere.
	changed: i64,
}

impl Seal {
	/// Reads the seal of `path`. Walks a folder, so call it off the runtime.
	pub fn of(path: &Path) -> std::io::Result<Self> {
		let metadata = std::fs::symlink_metadata(path)?;
		let mut seal = Self {
			is_dir: metadata.is_dir(),
			device: 0,
			inode: 0,
			entries: 0,
			size: 0,
			changed: 0,
		};
		#[cfg(unix)]
		{
			use std::os::unix::fs::MetadataExt;
			seal.device = metadata.dev();
			seal.inode = metadata.ino();
		}
		seal.add_tree(path, &metadata)?;
		Ok(seal)
	}

	fn add_tree(&mut self, path: &Path, metadata: &std::fs::Metadata) -> std::io::Result<()> {
		self.entries += 1;
		self.changed = self.changed.max(change_time(metadata));
		if metadata.is_dir() {
			for entry in std::fs::read_dir(path)? {
				let entry = entry?;
				// Links are part of the tree as themselves, never followed.
				self.add_tree(&entry.path(), &entry.path().symlink_metadata()?)?;
			}
		} else {
			self.size += metadata.len();
		}
		Ok(())
	}

	/// Checks that `path` is still what the job left there.
	pub fn check(&self, path: &Path) -> Result<(), String> {
		let current = match Self::of(path) {
			Ok(current) => current,
			Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
				return Err("it is no longer there".to_string())
			}
			Err(error) => return Err(format!("it cannot be read: {error}")),
		};
		if current.is_dir != self.is_dir
			|| current.device != self.device
			|| current.inode != self.inode
		{
			return Err("it was replaced by another item".to_string());
		}
		if current.entries != self.entries
			|| current.size != self.size
			|| current.changed != self.changed
		{
			return Err("it was changed after the operation".to_string());
		}
		Ok(())
	}
}

fn change_time(metadata: &std::fs::Metadata) -> i64 {
	#[cfg(unix)]
	{
		use std::os::unix::fs::MetadataExt;
		metadata
			.ctime()
			.saturating_mul(1_000_000_000)
			.saturating_add(metadata.ctime_nsec())
	}
	#[cfg(not(unix))]
	{
		metadata
			.modified()
			.ok()
			.and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
			.map_or(0, |since| {
				i64::try_from(since.as_nanos()).unwrap_or(i64::MAX)
			})
	}
}

/// Reads a seal without blocking the runtime.
pub async fn seal_of(path: &Path) -> std::io::Result<Seal> {
	let path = path.to_path_buf();
	tokio::task::spawn_blocking(move || Seal::of(&path))
		.await
		.map_err(std::io::Error::other)?
}

/// Checks a seal without blocking the runtime.
pub async fn check_seal(seal: &Seal, path: &Path) -> Result<(), String> {
	let (seal, path) = (seal.clone(), path.to_path_buf());
	tokio::task::spawn_blocking(move || seal.check(&path))
		.await
		.map_err(|error| error.to_string())?
}

/// A file moved aside so the job can write its replacement.
#[derive(Debug)]
pub struct Stash {
	original: PathBuf,
	stash: PathBuf,
	row: i32,
}

/// Writes the effects of one job.
pub struct Journal {
	db: DatabaseConnection,
	job_id: Uuid,
	operation: &'static str,
	next_seq: AtomicI32,
}

impl Journal {
	/// Opens the journal of the running job. `operation` is `copy`, `move`,
	/// `rename` or `delete`, shown when listing what can be undone.
	pub async fn open(ctx: &JobContext<'_>, operation: &'static str) -> Self {
		let db = ctx.library().jobs().database().conn().clone();
		let job_id = ctx.id().0;

		// Old stashes go when new work starts, so no timer is needed.
		let expired = purge_expired(&db, Utc::now() - Duration::days(RETENTION_DAYS)).await;
		if expired > 0 {
			ctx.log(format!("Removed {expired} expired undo records"));
		}

		// A resumed job continues its own numbering.
		let last = journal::Entity::find()
			.filter(journal::Column::JobId.eq(job_id.to_string()))
			.order_by_desc(journal::Column::Seq)
			.one(&db)
			.await
			.ok()
			.flatten()
			.map_or(0, |row| row.seq);

		Self {
			db,
			job_id,
			operation,
			next_seq: AtomicI32::new(last + 1),
		}
	}

	/// Records one effect. Returns the row, or `None` when it could not be
	/// written; the job goes on, and that item cannot be undone.
	pub async fn record(&self, effect: &Effect) -> Option<i32> {
		let encoded = match serde_json::to_string(effect) {
			Ok(encoded) => encoded,
			Err(error) => {
				// A name that is not valid Unicode has no JSON form.
				tracing::warn!(path = %effect.subject().display(), %error, "Could not journal a file effect");
				return None;
			}
		};
		let row = journal::ActiveModel {
			job_id: Set(self.job_id.to_string()),
			seq: Set(self.next_seq.fetch_add(1, Ordering::SeqCst)),
			operation: Set(self.operation.to_string()),
			effect: Set(encoded),
			created_at: Set(Utc::now()),
			undone_at: Set(None),
			..Default::default()
		};
		match row.insert(&self.db).await {
			Ok(row) => Some(row.id),
			Err(error) => {
				tracing::warn!(path = %effect.subject().display(), %error, "Could not journal a file effect");
				None
			}
		}
	}

	/// Moves the item at `path` aside before a job replaces it.
	///
	/// The stash is a folder next to the item, so the move is a rename on the
	/// same volume. Fails without touching `path` when the stash cannot be
	/// made or recorded.
	pub async fn stash(&self, path: &Path) -> std::io::Result<Stash> {
		let folder = path.with_file_name(format!(
			".wing-undo-{}",
			&self.job_id.simple().to_string()[..12]
		));
		tokio::fs::create_dir_all(&folder).await?;
		let stash = folder.join(Uuid::new_v4().simple().to_string());
		tokio::fs::rename(path, &stash).await?;

		let effect = Effect::Replaced {
			path: path.to_path_buf(),
			stash: stash.clone(),
		};
		match self.record(&effect).await {
			Some(row) => Ok(Stash {
				original: path.to_path_buf(),
				stash,
				row,
			}),
			None => {
				// Unrecorded, nothing would ever find or expire it.
				tokio::fs::rename(&stash, path).await?;
				let _ = tokio::fs::remove_dir(&folder).await;
				Err(std::io::Error::other(
					"the replaced file could not be recorded",
				))
			}
		}
	}

	/// Puts a stashed item back after the job failed to replace it.
	pub async fn unstash(&self, stash: Stash) {
		if tokio::fs::symlink_metadata(&stash.original).await.is_ok() {
			// Something is there after all; the stash stays for undo to sort out.
			return;
		}
		match tokio::fs::rename(&stash.stash, &stash.original).await {
			Ok(()) => {
				let _ = journal::Entity::delete_by_id(stash.row)
					.exec(&self.db)
					.await;
				if let Some(folder) = stash.stash.parent() {
					let _ = tokio::fs::remove_dir(folder).await;
				}
			}
			Err(error) => {
				tracing::error!(
					original = %stash.original.display(),
					stash = %stash.stash.display(),
					%error,
					"Could not put a replaced file back; it is still in the stash"
				);
			}
		}
	}
}

/// Effects of a job that were not undone yet, newest first.
pub async fn pending(db: &DatabaseConnection, job_id: Uuid) -> Result<Vec<(i32, Effect)>, String> {
	let rows = journal::Entity::find()
		.filter(journal::Column::JobId.eq(job_id.to_string()))
		.filter(journal::Column::UndoneAt.is_null())
		.order_by_desc(journal::Column::Seq)
		.all(db)
		.await
		.map_err(|error| error.to_string())?;
	rows.into_iter()
		.map(|row| {
			serde_json::from_str(&row.effect)
				.map(|effect| (row.id, effect))
				.map_err(|error| format!("Unreadable journal row {}: {error}", row.id))
		})
		.collect()
}

/// Marks one effect as reversed.
pub async fn mark_undone(db: &DatabaseConnection, row: i32) -> Result<(), String> {
	journal::ActiveModel {
		id: Set(row),
		undone_at: Set(Some(Utc::now())),
		..Default::default()
	}
	.update(db)
	.await
	.map(|_| ())
	.map_err(|error| error.to_string())
}

/// One operation in the journal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
	pub job_id: Uuid,
	pub operation: String,
	pub started_at: DateTime<Utc>,
	/// Effects recorded, and how many of them undo can still reverse.
	pub effects: u32,
	pub pending: u32,
	/// A path from the operation, to tell it apart in a list.
	pub example: PathBuf,
}

/// Recent operations, newest first.
pub async fn history(db: &DatabaseConnection, limit: usize) -> Result<Vec<Operation>, String> {
	let rows = journal::Entity::find()
		.order_by_desc(journal::Column::Id)
		.limit(HISTORY_SCAN_ROWS)
		.all(db)
		.await
		.map_err(|error| error.to_string())?;

	let mut operations: Vec<Operation> = Vec::new();
	for row in rows {
		let Ok(job_id) = Uuid::parse_str(&row.job_id) else {
			continue;
		};
		let effect: Option<Effect> = serde_json::from_str(&row.effect).ok();
		let undoable =
			row.undone_at.is_none() && !matches!(effect, Some(Effect::Merged { .. }) | None);
		let position = match operations.iter().position(|op| op.job_id == job_id) {
			Some(position) => position,
			None if operations.len() == limit => continue,
			None => {
				operations.push(Operation {
					job_id,
					operation: row.operation.clone(),
					started_at: row.created_at,
					effects: 0,
					pending: 0,
					example: PathBuf::new(),
				});
				operations.len() - 1
			}
		};
		let operation = &mut operations[position];
		operation.effects += 1;
		operation.pending += u32::from(undoable);
		// Rows come newest first, so the last one seen is the job's first.
		operation.started_at = row.created_at;
		if let Some(effect) = &effect {
			operation.example = effect.subject().to_path_buf();
		}
	}
	Ok(operations)
}

/// Removes journal rows older than `cutoff`, and the stashed files of the
/// ones that were never undone. Returns how many rows went.
pub async fn purge_expired(db: &DatabaseConnection, cutoff: DateTime<Utc>) -> u64 {
	let Ok(rows) = journal::Entity::find()
		.filter(journal::Column::CreatedAt.lt(cutoff))
		.all(db)
		.await
	else {
		return 0;
	};

	let mut removed = 0;
	for row in rows {
		if row.undone_at.is_none() {
			if let Ok(Effect::Replaced { stash, .. }) = serde_json::from_str(&row.effect) {
				let gone = match tokio::fs::remove_file(&stash).await {
					Ok(()) => true,
					Err(error) => error.kind() == std::io::ErrorKind::NotFound,
				};
				if !gone {
					// Keep the row: the stash is still there to be found.
					continue;
				}
				if let Some(folder) = stash.parent() {
					let _ = tokio::fs::remove_dir(folder).await;
				}
			}
		}
		if journal::Entity::delete_by_id(row.id).exec(db).await.is_ok() {
			removed += 1;
		}
	}
	removed
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn seal_accepts_an_untouched_tree_and_names_what_changed() {
		let root = tempfile::tempdir().unwrap();
		let folder = root.path().join("album");
		std::fs::create_dir_all(folder.join("inner")).unwrap();
		std::fs::write(folder.join("a.txt"), "aaaa").unwrap();
		std::fs::write(folder.join("inner/b.txt"), "bb").unwrap();

		let seal = Seal::of(&folder).unwrap();
		assert_eq!((seal.entries, seal.size), (4, 6));
		assert_eq!(seal.check(&folder), Ok(()));

		// Same size, new content: only the change time tells.
		std::thread::sleep(std::time::Duration::from_millis(20));
		std::fs::write(folder.join("inner/b.txt"), "BB").unwrap();
		assert_eq!(
			seal.check(&folder),
			Err("it was changed after the operation".to_string())
		);

		let seal = Seal::of(&folder).unwrap();
		std::fs::write(folder.join("new.txt"), "").unwrap();
		assert!(seal.check(&folder).is_err());

		let file = root.path().join("file.txt");
		std::fs::write(&file, "one").unwrap();
		let seal = Seal::of(&file).unwrap();
		std::fs::remove_file(&file).unwrap();
		assert_eq!(seal.check(&file), Err("it is no longer there".to_string()));
		std::fs::create_dir(&file).unwrap();
		assert_eq!(
			seal.check(&file),
			Err("it was replaced by another item".to_string())
		);
	}

	#[cfg(unix)]
	#[test]
	fn seal_does_not_follow_links() {
		let root = tempfile::tempdir().unwrap();
		let outside = root.path().join("outside");
		std::fs::create_dir(&outside).unwrap();
		std::fs::write(outside.join("big.bin"), vec![0u8; 4096]).unwrap();
		let folder = root.path().join("folder");
		std::fs::create_dir(&folder).unwrap();
		std::os::unix::fs::symlink(&outside, folder.join("link")).unwrap();
		std::os::unix::fs::symlink("missing", folder.join("dangling")).unwrap();

		let seal = Seal::of(&folder).unwrap();
		assert_eq!(seal.entries, 3);
		assert!(seal.size < 4096);
		// Changing the link's target is not a change to the folder.
		std::fs::write(outside.join("more.bin"), "x").unwrap();
		assert_eq!(seal.check(&folder), Ok(()));
	}

	#[test]
	fn effects_round_trip_through_their_stored_form() {
		let seal = Seal {
			is_dir: false,
			device: 1,
			inode: 2,
			entries: 1,
			size: 3,
			changed: 1_700_000_000_123_456_789,
		};
		for effect in [
			Effect::Created {
				path: "/a/b.txt".into(),
				seal: seal.clone(),
			},
			Effect::Moved {
				from: "/a/b.txt".into(),
				to: "/c/b.txt".into(),
				seal,
			},
			Effect::Replaced {
				path: "/a/b.txt".into(),
				stash: "/a/.wing-undo-1/2".into(),
			},
			Effect::Trashed {
				original: "/a/b.txt".into(),
				trash_id: None,
			},
			Effect::Merged { path: "/a".into() },
		] {
			let stored = serde_json::to_string(&effect).unwrap();
			assert_eq!(serde_json::from_str::<Effect>(&stored).unwrap(), effect);
		}
	}
}
